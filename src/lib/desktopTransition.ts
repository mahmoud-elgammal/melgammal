import { gsap } from "gsap";

/**
 * GSAP-driven curtain transition for entering/leaving /desktop. See
 * Head.astro's astro:before-preparation handler for how coverScreen() is
 * awaited before the actual page swap, and TransitionOverlay.astro for the
 * overlay markup (present on every page) plus the pre-paint FOUC guard
 * that makes the incoming page start already covered.
 */

function reducedMotion() {
    return window.matchMedia("(prefers-reduced-motion: reduce)").matches;
}

function getElements() {
    const overlay = document.getElementById("transition-overlay");
    const content = overlay?.querySelector<HTMLElement>(".transition-content");
    const label = document.getElementById("transition-label");
    const progress = document.getElementById("transition-progress");
    return { overlay, content, label, progress };
}

export function coverScreen(label: string): Promise<void> {
    const { overlay, content, label: labelEl, progress } = getElements();
    if (!overlay || !content || !labelEl) return Promise.resolve();

    labelEl.textContent = label;
    overlay.style.visibility = "visible";
    if (progress) progress.style.width = "0%";

    if (reducedMotion()) {
        gsap.set(overlay, { opacity: 1, scaleY: 1 });
        gsap.set(content, { opacity: 1 });
        if (progress) gsap.set(progress, { width: "100%" });
        return Promise.resolve();
    }

    return new Promise((resolve) => {
        const tl = gsap.timeline({ onComplete: resolve });
        tl.set(overlay, { transformOrigin: "bottom" })
            .to(overlay, {
                scaleY: 1,
                opacity: 1,
                duration: 0.55,
                ease: "power4.inOut",
            })
            .to(
                content,
                { opacity: 1, duration: 0.3, ease: "power2.out" },
                "-=0.2",
            );

        // The bar fills across the whole cover phase — like macOS's boot
        // progress bar, not an instant snap — using a slightly eased-out
        // curve so it reads as "loading" rather than a mechanical wipe.
        if (progress) {
            tl.to(
                progress,
                { width: "100%", duration: 0.7, ease: "power1.out" },
                0.1,
            );
        }
    });
}

export function revealScreen(): void {
    const { overlay, content } = getElements();
    if (!overlay || !content) return;

    const root = document.documentElement;
    const wasPrecovered = root.classList.contains("transition-precovered");
    root.classList.remove("transition-precovered");

    // Not every astro:after-swap follows a coverScreen() call on this same
    // document (most navigations never touch /desktop at all) — only run
    // the reveal if the overlay is actually showing something to reveal.
    const isCovering =
        wasPrecovered || getComputedStyle(overlay).visibility === "visible";
    if (!isCovering) return;

    if (reducedMotion()) {
        gsap.set(overlay, { opacity: 0, scaleY: 0, visibility: "hidden" });
        gsap.set(content, { opacity: 0 });
        return;
    }

    // The CSS-driven "precovered" state (from TransitionOverlay's pre-paint
    // script) is a class, not inline styles — pin the starting values down
    // explicitly before animating away from them, so GSAP isn't fighting a
    // stylesheet rule for control of the same properties.
    gsap.set(overlay, { opacity: 1, scaleY: 1, transformOrigin: "top" });
    gsap.set(content, { opacity: 1 });

    gsap
        .timeline({
            onComplete: () => {
                overlay.style.visibility = "hidden";
            },
        })
        .to(content, { opacity: 0, duration: 0.2, ease: "power2.in" })
        .to(
            overlay,
            { scaleY: 0, duration: 0.6, ease: "power4.inOut" },
            "-=0.05",
        );
}
