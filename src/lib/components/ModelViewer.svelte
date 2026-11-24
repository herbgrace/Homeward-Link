<script lang="ts">
  import { onMount } from 'svelte';
  import * as THREE from 'three';
  import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
  import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
  import type { CameraMode } from '$lib/utils/camera';

  interface Props {
    url?: string;
    rotationX?: number;
    rotationY?: number;
    rotationZ?: number;
    height?: number;
    scale?: number;
    order?: "XYZ" | "XZY" | "YXZ" | "YZX" | "ZXY" | "ZYX";
    camera?: CameraMode;
  }

  let {
    url = '/models/LowPoly/Rocket.glb', //Default to LowPoly Rocket
    rotationX = 0,
    rotationY = 0,
    rotationZ = 0,
    height = 0,
    scale = 1,
    order = "XYZ",
    camera: cameraMode = "chase",
  }: Props = $props();

  // Several viewers share the same model + textures, so let three.js cache the downloads
  THREE.Cache.enabled = true;

  // Altitude (m) at which the sky has fully faded to space
  const SPACE_ALTITUDE = 3000;

  // Sky palette, low altitude -> high altitude (sRGB hex)
  const SKY = {
    zenithLow: new THREE.Color('#2f6c9b'),
    zenithHigh: new THREE.Color('#020309'),
    horizonLow: new THREE.Color('#d8b59c'),
    horizonHigh: new THREE.Color('#c8642c'),
    // Atmospheric haze the ground fades into (also fills the sky below the horizon)
    hazeLow: new THREE.Color('#a9b6bb'),
    hazeHigh: new THREE.Color('#4d5a66'),
  };

  let container: HTMLDivElement;
  let model: THREE.Group | undefined;
  let ground: THREE.Group | undefined;

  onMount(() => {
    const width = container.clientWidth || 300;
    const viewHeight = container.clientHeight || 350;
    const textureLoader = new THREE.TextureLoader();

    // Scene, camera & renderer
    const scene = new THREE.Scene();
    scene.fog = new THREE.Fog(SKY.hazeLow.clone(), 80, 2500);

    const camera = new THREE.PerspectiveCamera(45, width / viewHeight, 0.1, 10000);
    camera.position.set(0, 2, 6);

    const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: false });
    renderer.setSize(width, viewHeight);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    container.appendChild(renderer.domElement);

    // Lighting
    scene.add(new THREE.AmbientLight(0xffffff, 1.2));

    const directionalLight = new THREE.DirectionalLight(0xffffff, 2.5);
    directionalLight.position.set(5, 10, 7);
    scene.add(directionalLight);

    const backLight = new THREE.DirectionalLight(0x90caf9, 1);
    backLight.position.set(-5, -5, -5);
    scene.add(backLight);

    // Sky dome - a vertical gradient that works from every camera angle and shifts toward space with altitude
    const skyUniforms = {
      zenith: { value: SKY.zenithLow.clone() },
      horizon: { value: SKY.horizonLow.clone() },
      haze: { value: SKY.hazeLow.clone() },
    };
    const sky = new THREE.Mesh(
      new THREE.SphereGeometry(4000, 32, 16),
      new THREE.ShaderMaterial({
        uniforms: skyUniforms,
        side: THREE.BackSide,
        depthWrite: false,
        vertexShader: `
          varying vec3 vDir;
          void main() {
            vDir = normalize(position);
            gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
          }`,
        fragmentShader: `
          uniform vec3 zenith;
          uniform vec3 horizon;
          uniform vec3 haze;
          varying vec3 vDir;
          void main() {
            float h = normalize(vDir).y;
            vec3 sky = mix(horizon, zenith, pow(max(h, 0.0), 0.5));
            vec3 c = mix(haze, sky, smoothstep(-0.01, 0.06, h));
            gl_FragColor = vec4(c, 1.0);
            #include <colorspace_fragment>
          }`,
      })
    );
    sky.renderOrder = -1;
    scene.add(sky);

    // Stars fade in as the rocket climbs
    const starPositions = new Float32Array(1500 * 3);
    for (let i = 0; i < 1500; i++) {
      const v = new THREE.Vector3().randomDirection();
      v.y = Math.abs(v.y) * 0.9 + 0.1;
      v.normalize().multiplyScalar(3500);
      starPositions.set([v.x, v.y, v.z], i * 3);
    }
    const starGeometry = new THREE.BufferGeometry();
    starGeometry.setAttribute('position', new THREE.BufferAttribute(starPositions, 3));
    const starMaterial = new THREE.PointsMaterial({ color: 0xffffff, size: 1.6, sizeAttenuation: false, transparent: true, opacity: 0, fog: false, depthWrite: false });
    const stars = new THREE.Points(starGeometry, starMaterial);
    scene.add(stars);

    // Ground: a large tiled field with a launch pad, moved down as altitude increases
    ground = new THREE.Group();
    const groundTexture = textureLoader.load('/img/textures/Grass.jpg');
    groundTexture.colorSpace = THREE.SRGBColorSpace;
    groundTexture.wrapS = groundTexture.wrapT = THREE.RepeatWrapping;
    groundTexture.repeat.set(2000, 2000);
    groundTexture.anisotropy = renderer.capabilities.getMaxAnisotropy();

    const groundMesh = new THREE.Mesh(
      new THREE.PlaneGeometry(30000, 30000),
      new THREE.MeshBasicMaterial({ map: groundTexture, side: THREE.DoubleSide })
    );
    groundMesh.rotation.x = -Math.PI / 2;
    ground.add(groundMesh);

    const pad = new THREE.Mesh(
      new THREE.CylinderGeometry(1.4, 1.5, 0.12, 48),
      new THREE.MeshStandardMaterial({ color: 0x8a8a8a, roughness: 0.9 })
    );
    pad.position.y = 0.06;
    ground.add(pad);
    scene.add(ground);

    // Distance the ground sits below the rocket's centre at launch; refined once the model loads
    let groundBase = -4;
    // Characteristic model size used to scale camera distances
    let modelRadius = 4;

    // Object the cameras will be looking at
    let camPivot: THREE.Group | undefined;
    const target = new THREE.Vector3(0, 0, 0);
    const box = new THREE.Box3();

    // Load GLB model
    const loader = new GLTFLoader();
    loader.load(
      url,
      (gltf) => {
        model = gltf.scene;
        model.scale.set(scale, scale, scale);
        const box = new THREE.Box3().setFromObject(model);
        const center = box.getCenter(new THREE.Vector3());
        const size = box.getSize(new THREE.Vector3());
        model.position.sub(center);
        model.traverse((o) => {
          const mesh = o as THREE.Mesh;
          if (mesh.isMesh) {
            const mats = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
            mats.forEach((m) => { if ("fog" in m) (m as THREE.MeshStandardMaterial).fog = false; });   // keep the rocket crisp at any distance
          }
        });
        const pivot = new THREE.Group();
        pivot.add(model);
        scene.add(pivot);
        camPivot = pivot;
        modelRadius = Math.max(size.x, size.y, size.z, 0.5);
        groundBase = -size.y / 2 - 0.05;
        pad.scale.setScalar(Math.max(modelRadius / 4, 0.4));
        updateRotation();
      },
      undefined,
      (error) => {
        console.error('An error occurred while loading the GLB model:', error);
      }
    );

    // Free-look controls (only active in "free" mode)
    let controls: OrbitControls | undefined;
    let activeMode: CameraMode | undefined;
    const clock = new THREE.Timer();

    function setMode(mode: CameraMode) {
      activeMode = mode;
      camera.up.set(0, 1, 0);
      camera.fov = 45;
      if (mode === "free") {
        const s = modelRadius / 4;
        camera.position.set(4 * s, 2.5 * s, 6 * s);
        controls = new OrbitControls(camera, renderer.domElement);
        controls.target.copy(target);
        controls.enableDamping = true;
        controls.minDistance = modelRadius * 0.5;
        controls.maxDistance = modelRadius * 40;
      } else {
        controls?.dispose();
        controls = undefined;
      }
      if (mode === "top") camera.up.set(0, 0, -1);
      camera.updateProjectionMatrix();
    }

    function updateTargetCenter(): THREE.Vector3 {
      if (!camPivot) {
        target.set(0, 0, 0);
        return target;
      }
      camPivot.updateWorldMatrix(true, true);
      box.setFromObject(camPivot);
      if (!box.isEmpty()) {
        box.getCenter(target);
      } else {
        target.set(0, 0, 0);
      }
      return target;
    }

    function updateCamera(mode: CameraMode, elapsed: number, groundY: number) {
      const s = modelRadius / 4;
      const center = updateTargetCenter();
      let fov = 45;
      switch (mode) {
        case "chase":
          camera.position.set(center.x, center.y + 2 * s, center.z + 6 * s);
          camera.lookAt(center);
          break;
        case "side":
          camera.position.set(center.x + 7.5 * s, center.y + 0.4 * s, center.z);
          camera.lookAt(center);
          break;
        case "top":
          camera.position.set(center.x, center.y + 9 * s, center.z + 0.001);
          camera.lookAt(center);
          break;
        case "orbit": {
          const a = elapsed * 0.3;
          camera.position.set(
            center.x + Math.sin(a) * 7 * s, 
            center.y + 1.5 * s, 
            center.z + Math.cos(a) * 7 * s);
          camera.lookAt(center);
          break;
        }
        case "ground": {
          // Stand on the field a short way from the pad and zoom to keep the rocket framed
          camera.position.set(center.x + 16 * s, groundY + 1.7, center.z + 12 * s);
          camera.lookAt(center);
          const dist = camera.position.distanceTo(center);
          fov = THREE.MathUtils.clamp(THREE.MathUtils.radToDeg(2 * Math.atan((modelRadius * 0.9) / dist)), 0.2, 50);
          break;
        }
        case "free":
          if (controls) {
            controls.target.copy(center);
            controls.update();
          }
          break;
      }
      if (camera.fov !== fov && mode !== "free") {
        camera.fov = fov;
        camera.updateProjectionMatrix();
      }
    }

    function updateEnvironment() {
      const groundY = groundBase - height;
      if (ground) ground.position.y = groundY;

      // Sky colour by altitude
      const t = THREE.MathUtils.smoothstep(height, 0, SPACE_ALTITUDE);
      skyUniforms.zenith.value.copy(SKY.zenithLow).lerp(SKY.zenithHigh, t);
      skyUniforms.horizon.value.copy(SKY.horizonLow).lerp(SKY.horizonHigh, Math.min(1, t * 1.4));
      skyUniforms.haze.value.copy(SKY.hazeLow).lerp(SKY.hazeHigh, t);
      // Push the haze out as we climb so the ground stays visible beneath the rocket
      const fog = scene.fog as THREE.Fog;
      fog.color.copy(skyUniforms.haze.value);
      fog.near = 80 + height * 0.8;
      fog.far = 2500 + height * 4;
      starMaterial.opacity = THREE.MathUtils.smoothstep(t, 0.35, 0.9);
      return groundY;
    }

    // Render loop
    let animationFrameId: number;
    function animate() {
      animationFrameId = requestAnimationFrame(animate);
      if (cameraMode !== activeMode) setMode(cameraMode);
      const groundY = updateEnvironment();
      updateCamera(cameraMode, clock.getElapsed(), groundY);
      sky.position.copy(camera.position);
      stars.position.copy(camera.position);
      renderer.render(scene, camera);
    }
    animate();

    // Track container size precisely
    const resizeObserver = new ResizeObserver((entries) => {
      for (let entry of entries) {
        const w = entry.contentRect.width;
        const h = entry.contentRect.height;
        if (w > 0 && h > 0) {
          camera.aspect = w / h;
          camera.updateProjectionMatrix();
          renderer.setSize(w, h);
        }
      }
    });
    resizeObserver.observe(container);

    // Cleanup on destroy - release the WebGL context so switching layouts doesn't leak contexts
    return () => {
      resizeObserver.disconnect();
      cancelAnimationFrame(animationFrameId);
      controls?.dispose();
      scene.traverse((o) => {
        const mesh = o as THREE.Mesh;
        mesh.geometry?.dispose?.();
      });
      groundTexture.dispose();
      if (renderer.domElement.parentElement === container) {
        container.removeChild(renderer.domElement);
      }
      renderer.dispose();
      renderer.forceContextLoss();
    };
  });

  function updateRotation() {
    if (model) {
      model.rotation.set(rotationX, rotationY, rotationZ, order);
    }
  }

  $effect(() => {
    rotationX;
    rotationY;
    rotationZ;
    order;
    updateRotation();
  });
</script>

<div bind:this={container} class="model-renderer" class:interactive={cameraMode === "free"}></div>

<style>
  .model-renderer {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 0;
    overflow: hidden;
    background: var(--viewport-bg);
  }

  /* Absolutely positioned so the canvas never props the container open while it shrinks */
  .model-renderer :global(canvas) {
    display: block;
    position: absolute;
    inset: 0;
  }

  .interactive { cursor: grab; }
  .interactive:active { cursor: grabbing; }
</style>
