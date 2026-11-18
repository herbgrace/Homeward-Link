<script lang="ts">
  import { onMount } from 'svelte';
  import * as THREE from 'three';
  import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';

  interface Props {
    url?: string;
    rotationX?: number;
    rotationY?: number;
    rotationZ?: number;
    height?: number;
    scale?: number;
    order?: "XYZ" | "XZY" | "YXZ" | "YZX" | "ZXY" | "ZYX";
  }

  let { 
    url = '/models/LowPoly/Rocket.glb', //Default to LowPoly Rocket
    rotationX = 0, 
    rotationY = 0, 
    rotationZ = 0,
    height = 0,
    scale = 1,
    order = "XYZ",
  }: Props = $props();

  let container: HTMLDivElement;
  let scene: THREE.Scene;
  let camera: THREE.PerspectiveCamera;
  let renderer: THREE.WebGLRenderer;
  let model: THREE.Group | undefined;
  let ground: THREE.Group | undefined;
  let background: THREE.Group | undefined;
  let animationFrameId: number;
  let pivot: THREE.Group | undefined;

  onMount(() => {
    const width = container.clientWidth || 300;
    const height = container.clientHeight || 350;
    const textureLoader = new THREE.TextureLoader();

    // Initialize Scene, Camera, & Renderer
    scene = new THREE.Scene();

    camera = new THREE.PerspectiveCamera(45, width / height, 0.1, 1000);
    camera.position.set(0, 2, 6);

    renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    renderer.setSize(width, height);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    container.appendChild(renderer.domElement);

    // Add Lighting
    const ambientLight = new THREE.AmbientLight(0xffffff, 1.2);
    scene.add(ambientLight);

    const directionalLight = new THREE.DirectionalLight(0xffffff, 2.5);
    directionalLight.position.set(5, 10, 7);
    scene.add(directionalLight);

    const backLight = new THREE.DirectionalLight(0x90caf9, 1);
    backLight.position.set(-5, -5, -5);
    scene.add(backLight);

    // Add Ground Objects
    ground = new THREE.Group();
    const groundTexture = textureLoader.load('/img/textures/Grass.jpg');

    const groudGeometry = new THREE.PlaneGeometry(20, 20);
    const groundMaterial = new THREE.MeshBasicMaterial({ 
      map: groundTexture,
      side: THREE.DoubleSide 
    });
    let groundMesh = new THREE.Mesh(groudGeometry, groundMaterial);
    groundMesh.rotateX(90);
    groundMesh.position.set(0, -4, 0);

    ground.add(groundMesh);
    scene.add(ground);

    // Add Background Objects
    background = new THREE.Group();
    const backgroundTexture = textureLoader.load('/img/textures/gradientBackground.png');

    const backgroundGeometry = new THREE.PlaneGeometry(20, 30);
    const backgroundMaterial = new THREE.MeshBasicMaterial({
      map: backgroundTexture,
      side: THREE.DoubleSide
    });
    let backgroundMesh = new THREE.Mesh(backgroundGeometry, backgroundMaterial);
    backgroundMesh.position.set(0, 3, -10);

    background.add(backgroundMesh);
    scene.add(background);

    // Load GLB Model
    const loader = new GLTFLoader();
    loader.load(
      url,
      (gltf) => {
        model = gltf.scene;
        model.scale.set(scale, scale, scale);
        const center = new THREE.Box3().setFromObject(model).getCenter(new THREE.Vector3());
        model.position.sub(center);
        pivot = new THREE.Group();
        pivot.add(model);
        scene.add(pivot);
        camera.lookAt(0, 0, 0);
        updateRotation();
        updateHeight();
      },
      undefined,
      (error) => {
        console.error('An error occurred while loading the GLB model:', error);
      }
    );

    // Render Loop
    function animate() {
      animationFrameId = requestAnimationFrame(animate);
      renderer.render(scene, camera);
    }
    animate();

    // Use ResizeObserver for precise container tracking
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

    // Cleanup on destroy
    return () => {
      resizeObserver.disconnect();
      cancelAnimationFrame(animationFrameId);
      if (renderer && renderer.domElement && container) {
        container.removeChild(renderer.domElement);
      }
      renderer?.dispose();
    };
  });

  // Function to apply rotation changes
  function updateRotation() {
    if (model) {
      model.rotation.set(rotationX, rotationY, rotationZ, order);
    }
  }

  function updateHeight() {
    // TODO - keep track of the starting height and only update based on that (in case starting height is not 0)
    if (ground) {
      ground.position.y = -height;
    }
    if (background) {
      background.position.y = -height * 0.002;
    }
  }

  $effect(() => {
    rotationX;
    rotationY;
    rotationZ;
    order;
    updateRotation();
  });

  $effect(() => {
    height;
    updateHeight();
  })
</script>

<!-- Give the container an explicit height class so it never collapses to 0 -->
<div bind:this={container} class="model-renderer"></div>

<style>
  .model-renderer {
    border: 2px solid;
    color: var(--accent-color);
  }
</style>