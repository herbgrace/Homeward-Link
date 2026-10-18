<script lang="ts">
  import { onMount } from 'svelte';
  import * as THREE from 'three';
  import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';

  interface Props {
    url?: string;
    rotationX?: number;
    rotationY?: number;
    rotationZ?: number;
    scale?: number;
  }

  let { 
    url = '/models/LowPoly/Rocket.glb', //Default to LowPoly Rocket
    rotationX = 0, 
    rotationY = 0, 
    rotationZ = 0,
    scale = 1 
  }: Props = $props();

  let container: HTMLDivElement;
  let scene: THREE.Scene;
  let camera: THREE.PerspectiveCamera;
  let renderer: THREE.WebGLRenderer;
  let model: THREE.Group | undefined;
  let animationFrameId: number;

  onMount(() => {
    const width = container.clientWidth || 300;
    const height = container.clientHeight || 350;

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

    // Load GLB Model
    const loader = new GLTFLoader();
    loader.load(
      url,
      (gltf) => {
        model = gltf.scene;
        model.scale.set(scale, scale, scale);
        scene.add(model);
        updateRotation();
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
      model.rotation.x = rotationX;
      model.rotation.y = rotationY;
      model.rotation.z = rotationZ;
    }
  }

  // React to prop changes from the parent component
  $effect(() => {
    console.log("Updating the model's rotation")
    rotationX;
    rotationY;
    rotationZ;
    updateRotation();
  });
</script>

<!-- Give the container an explicit height class so it never collapses to 0 -->
<div bind:this={container} class="w-full h-[350px] overflow-hidden"></div>