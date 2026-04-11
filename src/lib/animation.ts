/**
 * Animation Lifecycle Module
 *
 * Centralizes GSAP context management, view-transition cleanup, and
 * reduced-motion handling so every component can register and dispose
 * of animations through one consistent API.
 *
 * Usage in a component:
 *
 *   import { registerAnimation, prefersReducedMotion } from "@lib/animation";
 *
 *   document.addEventListener("astro:page-load", () => {
 *     registerAnimation("my-component", (gsap) => {
 *       // build animations here — context is auto-created
 *       gsap.from(".thing", { opacity: 0, y: 30 });
 *     });
 *   });
 *
 * Cleanup on view transitions is handled centrally — components do not
 * need to call revert() themselves.
 */

import { gsap } from "gsap";
import { ScrollTrigger } from "gsap/ScrollTrigger";

gsap.registerPlugin(ScrollTrigger);

type ContextBuilder = (g: typeof gsap, signal: AbortSignal) => void;

interface RegistryEntry {
    ctx: gsap.Context;
    controller: AbortController;
}

const registry = new Map<string, RegistryEntry>();
let reducedMotionMql: MediaQueryList | null = null;

/**
 * Returns true if the user has requested reduced motion. Live-updates if
 * the OS-level setting changes mid-session.
 */
export function prefersReducedMotion(): boolean {
    if (typeof window === "undefined") return false;
    if (!reducedMotionMql) {
        reducedMotionMql = window.matchMedia("(prefers-reduced-motion: reduce)");
    }
    return reducedMotionMql.matches;
}

/**
 * Register an animation block under a unique key. The builder receives
 * the gsap object and an AbortSignal — any DOM event listeners attached
 * via `addEventListener(type, fn, { signal })` will be removed
 * automatically when the registration is disposed (page swap, manual
 * dispose, or reduced-motion toggle).
 *
 * If a context with the same key already exists (e.g. from a previous
 * page render), it is disposed first so we never stack duplicate
 * animations or duplicate listeners.
 *
 * The builder should branch on `prefersReducedMotion()` and use instant
 * `gsap.set()` calls (or just early-return) for any state it would
 * normally animate.
 */
export function registerAnimation(key: string, build: ContextBuilder): void {
    // Always dispose any prior registration under this key first
    disposeAnimation(key);

    const controller = new AbortController();
    const ctx = gsap.context(() => build(gsap, controller.signal));
    registry.set(key, { ctx, controller });
}

/**
 * Manually dispose a single registered animation. Useful when a component
 * unmounts mid-session (e.g. modal close) without a full page swap.
 */
export function disposeAnimation(key: string): void {
    const entry = registry.get(key);
    if (entry) {
        entry.controller.abort();
        entry.ctx.revert();
        registry.delete(key);
    }
}

/**
 * Revert every registered animation context, abort every cleanup signal,
 * and kill any ScrollTriggers that escaped a context. Called
 * automatically before each Astro view transition.
 */
export function disposeAllAnimations(): void {
    registry.forEach((entry) => {
        entry.controller.abort();
        entry.ctx.revert();
    });
    registry.clear();

    // Defensive: kill any rogue triggers that escaped contexts
    ScrollTrigger.getAll().forEach((trigger) => trigger.kill());
}

/**
 * Refresh ScrollTrigger after layout changes (DOM ready, fonts loaded,
 * images decoded). Cheap to call multiple times.
 */
export function refreshScrollTriggers(): void {
    ScrollTrigger.refresh();
}

/**
 * Wire up the global lifecycle hooks. Call once from PageLayout's
 * top-level script. Idempotent — safe to call repeatedly.
 */
let lifecycleInstalled = false;
export function installAnimationLifecycle(): void {
    if (lifecycleInstalled || typeof document === "undefined") return;
    lifecycleInstalled = true;

    // Dispose everything before the next document swaps in
    document.addEventListener("astro:before-swap", () => {
        disposeAllAnimations();
    });

    // Live-react to OS reduced-motion toggle: kill all anims and force
    // any progress-based timelines to their resolved state.
    if (!reducedMotionMql) {
        reducedMotionMql = window.matchMedia("(prefers-reduced-motion: reduce)");
    }
    reducedMotionMql.addEventListener("change", (e) => {
        if (e.matches) {
            disposeAllAnimations();
        }
    });
}
