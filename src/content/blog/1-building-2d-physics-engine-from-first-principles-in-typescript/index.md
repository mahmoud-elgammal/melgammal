---
title: "Building a 2D Physics Engine from First Principles in TypeScript"
description: "A robust physics engine forms the backbone of countless interactive simulations, animations, and games."
date: "Mar 22 2024"
---

It’s the invisible hand that makes worlds feel alive and responsive. While many off-the-shelf libraries exist, building one from scratch is a rewarding journey that deepens your understanding of mathematics, physics, and software architecture. By writing an engine in TypeScript, you gain the benefits of type safety, world-class developer tooling, and seamless integration with the modern web ecosystem.

This guide will take you on a deep dive, providing:

- A flexible, modular architecture suitable for real-time applications.
- A first-principles look at the core physics, including the mathematical formulas for motion and collision.
- Explanations of collision detection and resolution, from simple checks to the powerful Separating Axis Theorem (SAT).
- Production-ready TypeScript code that is easy to extend with advanced features like constraints, joints, or continuous collision detection.

---

## Project Structure

A clean separation of concerns is critical for a maintainable engine. Organize your code into modules that handle distinct responsibilities, allowing you to easily expand or replace components later.

```
my-physics-engine/
├── src/
│   ├── math/
│   │   ├── Vector2.ts
│   │   └── Matrix2D.ts
│   ├── core/
│   │   ├── RigidBody.ts
│   │   ├── Shapes.ts
│   │   └── PhysicsEngine.ts
│   ├── collision/
│   │   ├── BroadPhase.ts
│   │   ├── NarrowPhase.ts
│   │   └── CollisionResolution.ts
│   └── index.ts
├── tsconfig.json
├── package.json
└── README.md
```

- **`math/`**: Contains utility classes for vectors, matrices, and geometry helpers. This is the mathematical foundation.
- **`core/`**: Defines the physical objects (rigid bodies), their geometric shapes, and the engine’s main update loop.
- **`collision/`**: Houses the algorithms for detecting and resolving collisions, from high-level spatial partitioning to precise, low-level impulse calculations.

---

## The Physics: Core Concepts and Data Structures

### RigidBody: The State of an Object

A `RigidBody` is the primary data structure that holds the complete physical state of an object at a given instant. For a comprehensive 2D engine, this includes both linear (translational) and angular (rotational) properties.

```typescript name=RigidBody.ts
export interface Shape {
  type: 'circle' | 'polygon';
  radius?: number;
  vertices?: Vector2[]; // Using a Vector2 class is cleaner
}

export interface RigidBody {
  id: string;

  // Linear motion
  position: Vector2;
  velocity: Vector2;
  force: Vector2; // Accumulated forces for the current frame
  mass: number;
  inverseMass: number; // Pre-calculated for efficiency (1/mass)

  // Angular motion
  angle: number; // Orientation in radians
  angularVelocity: number;
  torque: number; // Accumulated torque
  inertia: number;
  inverseInertia: number; // Pre-calculated (1/inertia)

  // Material properties
  restitution: number; // "Bounciness" factor (0 to 1)
  staticFriction: number;
  dynamicFriction: number;

  shape: Shape;
}
```

The core of the simulation is to evolve this state over time based on physical laws. Newton's second law governs motion:

- **Linear Motion**: `F = ma` (Force = mass × acceleration). We can rearrange this to find the acceleration: `a = F / m`.
- **Rotational Motion**: `τ = Iα` (Torque = moment of inertia × angular acceleration). Similarly, angular acceleration is: `α = τ / I`.

The `inverseMass` and `inverseInertia` are pre-calculated because multiplication is computationally cheaper than division. For static objects (e.g., the ground), we can simulate infinite mass by setting `inverseMass` and `inverseInertia` to zero. This gracefully handles divisions by zero and prevents static objects from moving.

---

## The Heartbeat: Integration and the Update Loop

The engine's "heartbeat" is the update loop, which advances the simulation forward in time. This process is called **numerical integration**. Since we cannot solve the equations of motion continuously, we approximate them in discrete time steps (`dt`).

### Numerical Integrators

The choice of integrator impacts stability and accuracy.

1.  **Explicit Euler**: The simplest method. It calculates the state at the next time step using only the state at the current time step. It's easy to implement but can be unstable and inaccurate, especially with large time steps.

    *   `velocity(t + dt) = velocity(t) + acceleration(t) * dt`
    *   `position(t + dt) = position(t) + velocity(t) * dt`

2.  **Semi-Implicit Euler (or Symplectic Euler)**: A significant improvement in stability with minimal extra cost. It calculates the new velocity first and then uses that *new* velocity to calculate the new position. This prevents the "energy explosion" that can occur with Explicit Euler.

    *   `velocity(t + dt) = velocity(t) + acceleration(t) * dt`
    *   `position(t + dt) = position(t) + velocity(t + dt) * dt`  **(This is the key difference)**

### The Fixed-Timestep Loop

For a stable and deterministic simulation, it's crucial to use a **fixed time step**. This decouples the physics simulation from the rendering frame rate. If the game renders at a variable FPS, the physics will still behave consistently.

```typescript name=PhysicsEngine.ts
import { RigidBody } from './RigidBody';
import { circleCircleCollision } from '../collision/NarrowPhase';
import { resolveCollision } from '../collision/CollisionResolution';

export class PhysicsEngine {
  private bodies: RigidBody[] = [];
  private gravity = { x: 0, y: 9.81 };

  private lastTime = 0;
  private accumulator = 0;
  private readonly fixedDeltaTime = 1 / 60; // Run physics at 60 Hz

  addBody(body: RigidBody) {
    this.bodies.push(body);
  }

  gameLoop(currentTime: number) {
    const frameTime = (currentTime - this.lastTime) / 1000;
    this.lastTime = currentTime;
    this.accumulator += frameTime;

    while (this.accumulator >= this.fixedDeltaTime) {
      this.update(this.fixedDeltaTime); // Update with fixed step
      this.accumulator -= this.fixedDeltaTime;
    }

    // requestAnimationFrame(() => this.gameLoop(performance.now()));
  }

  update(dt: number) {
    // 1. Apply forces and integrate
    for (const body of this.bodies) {
      if (body.inverseMass === 0) continue;

      // Apply gravity
      body.velocity.y += this.gravity.y * dt;

      // Semi-Implicit Euler integration
      body.position.x += body.velocity.x * dt;
      body.position.y += body.velocity.y * dt;
      body.angle += body.angularVelocity * dt;
    }

    // 2. Collision detection (broad and narrow phase)
    // 3. Collision resolution
  }
}
```

---

## Collision Detection: Are We Touching?

Collision detection is a multi-stage process designed to be efficient.

### Broad-Phase: Finding Potential Collisions

Checking every object against every other object is an `O(n²)` operation, which is too slow for large scenes. The broad-phase uses a fast, approximate method to cull pairs of objects that are obviously not colliding. Spatial partitioning structures like **Quadtrees** or **Spatial Grids** are perfect for this. They quickly identify objects that are near each other.

### Narrow-Phase: Precise Collision Checks

Once the broad-phase gives us a small list of candidate pairs, we perform exact collision checks.

#### Separating Axis Theorem (SAT)

For convex polygons, the **Separating Axis Theorem** is a powerful and versatile algorithm. The core idea is:

> If you can find a single line (an "axis") onto which the two shapes' shadows do not overlap, then the shapes are not colliding. If you cannot find such a separating axis, they are colliding.

The only axes you need to test are the ones perpendicular to each edge of the polygons.

The process is:
1.  Get all axes to test (the normals of the edges of both polygons).
2.  For each axis:
    a. "Project" both shapes onto the axis. This results in two 1D intervals (a min and max value). The projection is calculated using the dot product: `projection = Vector2.dot(vertex, axis)`.
    b. Check if the two intervals overlap.
    c. If they do not overlap, you have found a separating axis. The shapes are not colliding, and you can stop immediately.
3.  If you test all axes and find an overlap on every single one, the shapes are colliding.

The SAT can also give you the **Minimum Translation Vector (MTV)**—the smallest vector needed to push the shapes apart, which provides the collision normal and penetration depth needed for resolution.

```typescript name=NarrowPhase.ts
import { RigidBody } from '../core/RigidBody';

// (circleCircleCollision is simple, so let's focus on a conceptual SAT)

export function polygonPolygonCollision(a: RigidBody, b: RigidBody): CollisionManifold | null {
  // 1. Get axes from both polygons
  // 2. Loop through axes
  //    a. Project both polygons onto the current axis
  //    b. Check for overlap. If no overlap, return null.
  //       Keep track of the minimum overlap and the axis it occurred on.
  // 3. If all axes have overlaps, create and return a CollisionManifold
  //    containing the collision normal (the axis of minimum overlap) and
  //    penetration depth.

  return null; // Placeholder
}
```

---

## Collision Resolution: Responding to Contact

When two bodies collide, we must adjust their velocities and positions to produce a realistic response. This is done by applying an **impulse**, which is a large force applied over a very short time.

The formula for the impulse scalar (`j`) for two colliding bodies (a and b) is:

`j = -(1 + e) * (v_rel · n) / (1/m_a + 1/m_b + (r_ap ⊥ n)²/I_a + (r_bp ⊥ n)²/I_b)`

Let's break this down:
- `e`: The coefficient of restitution (bounciness). `e = Math.min(a.restitution, b.restitution)`.
- `v_rel`: The relative velocity at the point of contact.
- `n`: The collision normal (a unit vector pointing from body A to body B, provided by the narrow-phase).
- `m_a`, `m_b`: The masses of the bodies.
- `I_a`, `I_b`: The moments of inertia of the bodies.
- `r_ap`, `r_bp`: The vectors from each body's center of mass to the collision contact point.
- `r ⊥ n`: The perpendicular component of `r` relative to `n`, related to the cross product in 2D.

Once `j` is calculated, the impulse vector is `J = j * n`. This impulse is then applied to change the linear and angular velocities:

```typescript name=CollisionResolution.ts
import { RigidBody } from '../core/RigidBody';
import { Vector2 } from '../math/Vector2'; // Assuming a Vector2 class

export function resolveCollision(manifold: CollisionManifold) {
  const { a, b, normal, penetration } = manifold;

  // Calculate relative velocity
  const rv = b.velocity.clone().subtract(a.velocity);

  // Calculate relative velocity in terms of the normal direction
  const velAlongNormal = rv.dot(normal);

  // Do not resolve if velocities are separating
  if (velAlongNormal > 0) return;

  // Calculate restitution
  const e = Math.min(a.restitution, b.restitution);

  // Calculate impulse scalar
  const j = -(1 + e) * velAlongNormal / (a.inverseMass + b.inverseMass);

  // Apply impulse
  const impulse = normal.clone().multiply(j);
  a.velocity.subtract(impulse.clone().multiply(a.inverseMass));
  b.velocity.add(impulse.clone().multiply(b.inverseMass));

  // (A full implementation would also calculate and apply rotational impulse)
}
```

### Positional Correction

To prevent objects from sinking into each other due to numerical inaccuracies, a positional correction step is often applied. This moves the objects apart along the collision normal by a small percentage of the penetration depth.

```typescript
function positionalCorrection(manifold: CollisionManifold) {
    const { a, b, normal, penetration } = manifold;
    const percent = 0.2; // Usually 20-80%
    const slop = 0.01; // Penetration allowance
    const correctionAmount = Math.max(penetration - slop, 0) / (a.inverseMass + b.inverseMass) * percent;
    const correction = normal.clone().multiply(correctionAmount);

    a.position.subtract(correction.clone().multiply(a.inverseMass));
    b.position.add(correction.clone().multiply(b.inverseMass));
}
```
