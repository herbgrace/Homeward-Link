<script lang="ts">
  import { onMount } from 'svelte';
  import * as THREE from 'three';
  import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';

  // Define TypeScript interface for component props
  interface Props {
    url?: string;
    rotationX?: number;
    rotationY?: number;
    rotationZ?: number;
    scale?: number;
  }

  let { 
    url = '/models/rocket.glb', 
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
    // 1. Initialize Scene, Camera, & Renderer
    scene = new THREE.Scene();

    camera = new THREE.PerspectiveCamera(
      45,
      container.clientWidth / container.clientHeight,
      0.1,
      1000
    );
    camera.position.set(0, 2, 6);

    renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    renderer.setSize(container.clientWidth, container.clientHeight);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    container.appendChild(renderer.domElement);

    // 2. Add Lighting
    const ambientLight = new THREE.AmbientLight(0xffffff, 1.2);
    scene.add(ambientLight);

    const directionalLight = new THREE.DirectionalLight(0xffffff, 2.5);
    directionalLight.position.set(5, 10, 7);
    scene.add(directionalLight);

    const backLight = new THREE.DirectionalLight(0x90caf9, 1);
    backLight.position.set(-5, -5, -5);
    scene.add(backLight);

    // 3. Load GLB Model
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

    // 4. Render Loop
    function animate() {
      animationFrameId = requestAnimationFrame(animate);
      renderer.render(scene, camera);
    }
    animate();

    // 5. Handle Window Resize
    function handleResize() {
      if (!container) return;
      camera.aspect = container.clientWidth / container.clientHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(container.clientWidth, container.clientHeight);
    }
    window.addEventListener('resize', handleResize);

    // Cleanup on destroy
    return () => {
      window.removeEventListener('resize', handleResize);
      cancelAnimationFrame(animationFrameId);
      if (renderer && renderer.domElement) {
        container.removeChild(renderer.domElement);
      }
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
    // Track dependencies explicitly for Svelte 5 effect rune
    rotationX;
    rotationY;
    rotationZ;
    updateRotation();
  });
</script>

<div bind:this={container} class="w-full h-full min-h-[350px] overflow-hidden"></div>