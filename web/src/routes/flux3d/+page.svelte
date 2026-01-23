<script lang="ts">
  import { onMount } from 'svelte';
  import * as THREE from 'three';
  import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls';
  
  let container: HTMLDivElement;
  
  const datasets = [
    {
      name: 'Sacred Virtues',
      data: [
        { pos: 0, label: 'Freedom', e: 0.7, l: 0.8, p: 0.6 },
        { pos: 1, label: 'Joy', e: 0.6, l: 0.4, p: 0.9 },
        { pos: 2, label: 'Peace', e: 0.6, l: 0.5, p: 0.8 },
        { pos: 3, label: 'Love', e: 0.7, l: 0.5, p: 0.95 },
        { pos: 4, label: 'Beauty', e: 0.6, l: 0.6, p: 0.8 },
        { pos: 5, label: 'Courage', e: 0.95, l: 0.6, p: 0.4 },
        { pos: 6, label: 'Truth', e: 0.85, l: 0.95, p: 0.5 },
        { pos: 7, label: 'Justice', e: 0.9, l: 0.7, p: 0.5 },
        { pos: 8, label: 'Wisdom', e: 0.85, l: 0.95, p: 0.5 },
        { pos: 9, label: 'Creation', e: 0.9, l: 0.6, p: 0.5 },
      ]
    }
  ];
  
  let currentDataset = 0;
  
  onMount(() => {
    // Scene setup
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x0a0a1a);
    
    const camera = new THREE.PerspectiveCamera(75, container.clientWidth / container.clientHeight, 0.1, 1000);
    camera.position.set(0, 0, 15);
    
    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(container.clientWidth, container.clientHeight);
    container.appendChild(renderer.domElement);
    
    const controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    
    // Lighting
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.5);
    scene.add(ambientLight);
    
    const pointLight = new THREE.PointLight(0xffffff, 1);
    pointLight.position.set(10, 10, 10);
    scene.add(pointLight);
    
    function createFluxVisualization(data: any[]) {
      const group = new THREE.Group();
      const radius = 8;
      
      // Sacred triangle (3-6-9)
      const triangleGeometry = new THREE.BufferGeometry();
      const pos3 = getPosition(3, radius);
      const pos6 = getPosition(6, radius);
      const pos9 = getPosition(9, radius);
      
      const trianglePoints = new Float32Array([
        ...pos3, ...pos6,
        ...pos6, ...pos9,
        ...pos9, ...pos3
      ]);
      
      triangleGeometry.setAttribute('position', new THREE.BufferAttribute(trianglePoints, 3));
      const triangleMaterial = new THREE.LineBasicMaterial({ color: 0xffffff, linewidth: 3 });
      const triangle = new THREE.LineSegments(triangleGeometry, triangleMaterial);
      group.add(triangle);
      
      // Doubling sequence star (1-2-4-8-7-5-1)
      const starSequence = [1, 2, 4, 8, 7, 5, 1];
      const starGeometry = new THREE.BufferGeometry();
      const starPoints: number[] = [];
      
      for (let i = 0; i < starSequence.length - 1; i++) {
        const p1 = getPosition(starSequence[i], radius);
        const p2 = getPosition(starSequence[i + 1], radius);
        starPoints.push(...p1, ...p2);
      }
      
      starGeometry.setAttribute('position', new THREE.BufferAttribute(new Float32Array(starPoints), 3));
      const starMaterial = new THREE.LineBasicMaterial({ color: 0x444466, linewidth: 1 });
      const star = new THREE.LineSegments(starGeometry, starMaterial);
      group.add(star);
      
      // Outer circle
      const circleGeometry = new THREE.RingGeometry(radius - 0.1, radius + 0.1, 64);
      const circleMaterial = new THREE.MeshBasicMaterial({ color: 0x333366, side: THREE.DoubleSide });
      const circle = new THREE.Mesh(circleGeometry, circleMaterial);
      group.add(circle);
      
      // Data points as spheres
      data.forEach(point => {
        const pos = getPosition(point.pos, radius);
        
        // Determine dominant channel color
        const color = getDominantColor(point.e, point.l, point.p);
        const sphereGeometry = new THREE.SphereGeometry(0.4, 32, 32);
        const sphereMaterial = new THREE.MeshPhongMaterial({ 
          color,
          emissive: color,
          emissiveIntensity: 0.3
        });
        const sphere = new THREE.Mesh(sphereGeometry, sphereMaterial);
        sphere.position.set(...pos);
        
        // Add glow for sacred positions
        if ([3, 6, 9].includes(point.pos)) {
          const glowGeometry = new THREE.SphereGeometry(0.6, 32, 32);
          const glowMaterial = new THREE.MeshBasicMaterial({
            color,
            transparent: true,
            opacity: 0.3
          });
          const glow = new THREE.Mesh(glowGeometry, glowMaterial);
          glow.position.set(...pos);
          group.add(glow);
        }
        
        group.add(sphere);
        
        // Label (using sprite)
        const canvas = document.createElement('canvas');
        const context = canvas.getContext('2d')!;
        canvas.width = 256;
        canvas.height = 64;
        context.fillStyle = '#ffffff';
        context.font = 'Bold 32px Arial';
        context.textAlign = 'center';
        context.fillText(point.label, 128, 40);
        
        const texture = new THREE.CanvasTexture(canvas);
        const spriteMaterial = new THREE.SpriteMaterial({ map: texture });
        const sprite = new THREE.Sprite(spriteMaterial);
        sprite.position.set(pos[0] * 1.2, pos[1] * 1.2, pos[2]);
        sprite.scale.set(2, 0.5, 1);
        group.add(sprite);
      });
      
      return group;
    }
    
    function getPosition(pos: number, radius: number): [number, number, number] {
      // Position 9 at top (12 o'clock), clockwise
      const angle = ((9 - pos) / 9) * Math.PI * 2 - Math.PI / 2;
      return [
        Math.cos(angle) * radius,
        Math.sin(angle) * radius,
        0
      ];
    }
    
    function getDominantColor(e: number, l: number, p: number): number {
      if (e > l && e > p) return 0xff4444; // Red - Ethos
      if (l > p) return 0x4444ff; // Blue - Logos
      return 0.44ff44; // Green - Pathos
    }
    
    let fluxGroup = createFluxVisualization(datasets[currentDataset].data);
    scene.add(fluxGroup);
    
    // Animation loop
    function animate() {
      requestAnimationFrame(animate);
      controls.update();
      fluxGroup.rotation.z += 0.002; // Slow rotation
      renderer.render(scene, camera);
    }
    animate();
    
    // Resize handler
    function onResize() {
      camera.aspect = container.clientWidth / container.clientHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(container.clientWidth, container.clientHeight);
    }
    window.addEventListener('resize', onResize);
    
    return () => {
      window.removeEventListener('resize', onResize);
      renderer.dispose();
    };
  });
</script>

<div class="page">
  <h1>🌀 Flux Matrix 3D</h1>
  <p class="subtitle">Interactive Sacred Geometry • Vortex Math Pattern</p>
  
  <div class="viewer" bind:this={container}></div>
  
  <div class="legend">
    <h3>ELP Channels</h3>
    <div class="item"><span class="dot red"></span> Ethos (Character)</div>
    <div class="item"><span class="dot blue"></span> Logos (Logic)</div>
    <div class="item"><span class="dot green"></span> Pathos (Emotion)</div>
    <h3 style="margin-top: 1rem;">Sacred Triangle</h3>
    <p style="font-size: 0.9em;">Positions 3-6-9</p>
  </div>
</div>

<style>
  .page {
    min-height: 100vh;
    background: linear-gradient(135deg, #0a0a1a 0%, #1a1a2e 100%);
    color: white;
    padding: 2rem;
  }
  
  h1 {
    text-align: center;
    font-size: 2.5em;
    margin: 0;
    background: linear-gradient(90deg, #ff4444 0%, #4444ff 50%, #44ff44 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  
  .subtitle {
    text-align: center;
    color: #aaa;
    margin: 0.5rem 0 2rem;
  }
  
  .viewer {
    width: 100%;
    height: 600px;
    border-radius: 12px;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  }
  
  .legend {
    max-width: 800px;
    margin: 2rem auto;
    padding: 1.5rem;
    background: rgba(0, 0, 0, 0.3);
    border-radius: 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }
  
  .legend h3 {
    color: #ffaa44;
    margin: 0 0 1rem 0;
  }
  
  .item {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin: 0.5rem 0;
  }
  
  .dot {
    width: 16px;
    height: 16px;
    border-radius: 50%;
  }
  
  .red { background: #ff4444; box-shadow: 0 0 10px #ff4444; }
  .blue { background: #4444ff; box-shadow: 0 0 10px #4444ff; }
  .green { background: #44ff44; box-shadow: 0 0 10px #44ff44; }
</style>
