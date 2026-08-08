import { prefersReducedMotion } from "@lib/animation";

/**
 * Renders a heading element's text as a WebGL texture, in small pixel
 * blocks that follow the cursor's movement direction — the hero title and
 * footer wordmark both use this. Falls back to the plain, fully visible
 * source element if WebGL is unavailable or reduced motion is requested.
 *
 * Caller owns the DOM entrance animation (letter reveal, etc.) and only
 * constructs this once that's finished, since the WebGL layer takes over
 * rendering by setting the source element's color to transparent.
 */

const VERTEX_SRC = `
    attribute vec2 aPosition;
    varying vec2 vUv;
    void main() {
        vUv = aPosition * 0.5 + 0.5;
        gl_Position = vec4(aPosition, 0.0, 1.0);
    }
`;

const FRAGMENT_SRC = `
    precision highp float;
    varying vec2 vUv;
    uniform sampler2D uTexture;
    uniform vec2 uResolution;
    uniform float uBlockSize;
    uniform vec2 uMouse;
    uniform vec2 uVelocityPx;
    uniform float uTime;

    float hash(vec2 p) {
        return fract(sin(dot(p, vec2(41.3, 289.1))) * 43758.5453);
    }

    void main() {
        vec2 fragPx = vUv * uResolution;
        vec2 blockCoord = floor(fragPx / uBlockSize);
        float r1 = hash(blockCoord);

        float idle = sin(uTime * 1.5 + r1 * 6.2831) * 0.4;
        vec2 idleOffsetPx = vec2(idle, idle * 0.5);

        // Blocks near the cursor displace along the direction the mouse is
        // actually moving (not random) — a deliberate "push" that follows
        // the pointer instead of a jittery glitch.
        vec2 hoverOffsetPx = vec2(0.0);
        if (uMouse.x >= 0.0) {
            vec2 mousePx = uMouse * uResolution;
            vec2 blockCenterPx = (blockCoord + 0.5) * uBlockSize;
            float dist = length(blockCenterPx - mousePx);
            float influence = smoothstep(240.0, 0.0, dist);
            hoverOffsetPx = uVelocityPx * influence;
        }

        vec2 offsetPx = idleOffsetPx + hoverOffsetPx;
        vec2 sampleUv = (fragPx + offsetPx) / uResolution;

        gl_FragColor = texture2D(uTexture, sampleUv);
    }
`;

function compileShader(
    gl: WebGLRenderingContext,
    type: number,
    source: string,
): WebGLShader | null {
    const shader = gl.createShader(type);
    if (!shader) return null;
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        gl.deleteShader(shader);
        return null;
    }
    return shader;
}

export class PixelTextFX {
    private el: HTMLElement;
    private lines: string[];
    private canvas: HTMLCanvasElement;
    private gl: WebGLRenderingContext;
    private program: WebGLProgram;
    private texture: WebGLTexture;
    private textCanvas = document.createElement("canvas");
    private textCtx: CanvasRenderingContext2D;
    private uniforms: Record<string, WebGLUniformLocation | null> = {};
    private dpr = Math.min(window.devicePixelRatio || 1, 2);
    private mouse = { x: -1, y: -1 };
    private targetMouse = { x: -1, y: -1 };
    private prevMouse = { x: -1, y: -1 };
    private velocity = { x: 0, y: 0 };
    private rafId: number | null = null;
    private startTime = performance.now();
    private reduced: boolean;
    private themeObserver: MutationObserver | null = null;
    private colors: { light: string; dark: string };

    constructor(
        el: HTMLElement,
        lines: string[],
        canvas: HTMLCanvasElement,
        gl: WebGLRenderingContext,
        signal: AbortSignal,
        colors: { light: string; dark: string } = {
            light: "#0a0a0a",
            dark: "#f8f8f6",
        },
    ) {
        this.el = el;
        this.lines = lines;
        this.canvas = canvas;
        this.gl = gl;
        this.colors = colors;
        this.reduced = prefersReducedMotion();
        this.textCtx = this.textCanvas.getContext("2d")!;

        const program = this.createProgram();
        if (!program) throw new Error("shader program failed");
        this.program = program;

        const texture = gl.createTexture();
        if (!texture) throw new Error("texture creation failed");
        this.texture = texture;

        this.setupGeometry();
        this.layout();

        window.addEventListener("resize", () => this.layout(), {
            signal,
            passive: true,
        });

        if (!this.reduced) {
            canvas.addEventListener(
                "mousemove",
                (e) => {
                    const rect = canvas.getBoundingClientRect();
                    this.targetMouse.x = (e.clientX - rect.left) / rect.width;
                    // Flipped: gl_Position/vUv are bottom-up in GLSL, but
                    // clientY is top-down — without this the hover offset
                    // lands mirrored vertically from the cursor.
                    this.targetMouse.y =
                        1 - (e.clientY - rect.top) / rect.height;
                },
                { signal, passive: true },
            );
            canvas.addEventListener(
                "mouseleave",
                () => {
                    this.targetMouse.x = -1;
                    this.targetMouse.y = -1;
                },
                { signal, passive: true },
            );
        }

        this.themeObserver = new MutationObserver(() => this.layout());
        this.themeObserver.observe(document.documentElement, {
            attributes: true,
            attributeFilter: ["class"],
        });

        signal.addEventListener("abort", () => this.stop());

        // Paint the first frame before revealing the canvas, so there's
        // never a blank tick between hiding the source element and the
        // shader actually having something to show.
        this.render();
        el.style.color = "transparent";
        canvas.style.opacity = "1";

        // The initial render() above already painted a static frame;
        // reduced motion stops there. Otherwise, keep rendering for the
        // idle shimmer and cursor-follow interaction.
        if (!this.reduced) this.loop();
    }

    private createProgram(): WebGLProgram | null {
        const { gl } = this;
        const vertexShader = compileShader(gl, gl.VERTEX_SHADER, VERTEX_SRC);
        const fragmentShader = compileShader(
            gl,
            gl.FRAGMENT_SHADER,
            FRAGMENT_SRC,
        );
        if (!vertexShader || !fragmentShader) return null;

        const program = gl.createProgram();
        if (!program) return null;
        gl.attachShader(program, vertexShader);
        gl.attachShader(program, fragmentShader);
        gl.linkProgram(program);
        if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
            return null;
        }

        gl.useProgram(program);
        [
            "uTexture",
            "uResolution",
            "uBlockSize",
            "uMouse",
            "uVelocityPx",
            "uTime",
        ].forEach((name) => {
            this.uniforms[name] = gl.getUniformLocation(program, name);
        });

        return program;
    }

    private setupGeometry() {
        const { gl, program } = this;
        const positions = new Float32Array([-1, -1, 3, -1, -1, 3]);
        const buffer = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, buffer);
        gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);

        const aPosition = gl.getAttribLocation(program, "aPosition");
        gl.enableVertexAttribArray(aPosition);
        gl.vertexAttribPointer(aPosition, 2, gl.FLOAT, false, 0, 0);
    }

    // Draws the current text into the offscreen 2D canvas (matching the
    // source element's live computed font/size/color), then uploads it as
    // the WebGL texture and resizes/positions the canvas over the element.
    private layout() {
        const { gl } = this;
        const cs = getComputedStyle(this.el);
        const rect = this.el.getBoundingClientRect();
        const wrapRect = this.el.parentElement!.getBoundingClientRect();

        const width = Math.max(1, rect.width);
        const height = Math.max(1, rect.height);
        const fontSize = parseFloat(cs.fontSize);
        const lineHeight = parseFloat(cs.lineHeight) || fontSize * 1.05;

        this.canvas.style.left = `${rect.left - wrapRect.left}px`;
        this.canvas.style.top = `${rect.top - wrapRect.top}px`;
        this.canvas.style.width = `${width}px`;
        this.canvas.style.height = `${height}px`;
        this.canvas.width = Math.round(width * this.dpr);
        this.canvas.height = Math.round(height * this.dpr);

        this.textCanvas.width = this.canvas.width;
        this.textCanvas.height = this.canvas.height;

        const ctx = this.textCtx;
        ctx.clearRect(0, 0, this.textCanvas.width, this.textCanvas.height);
        ctx.scale(this.dpr, this.dpr);
        ctx.font = `${cs.fontWeight} ${fontSize}px ${cs.fontFamily}`;
        // NOT cs.color: once the source element is hidden (color:
        // transparent, set at the end of the constructor), getComputedStyle()
        // would read that transparent value back on every later layout()
        // call — e.g. on resize, or when the theme MutationObserver fires —
        // silently drawing invisible text into the texture and leaving the
        // canvas permanently blank. Derive the real color from the current
        // theme instead.
        const isDark = document.documentElement.classList.contains("dark");
        ctx.fillStyle = isDark ? this.colors.dark : this.colors.light;
        ctx.textBaseline = "alphabetic";
        // Match the element's letter-spacing so measureText() wrap points
        // line up with how the browser actually wraps the real text.
        if ("letterSpacing" in ctx) {
            (ctx as any).letterSpacing = cs.letterSpacing;
        }

        // The source element word-wraps each line with flexbox at whatever
        // the real viewport width is, so a fixed line-break assumption
        // drifts out of sync (e.g. bigger font sizes wrap onto more lines).
        // Re-wrap the same lines against this canvas's own width so the
        // drawn text always matches the source element. Small safety
        // margin: if canvas measurement and real flexbox layout round a
        // hair differently, better to wrap one word early than to overflow
        // the canvas and clip visibly.
        const wrapWidth = width - 4;
        const rows: string[] = [];
        this.lines.forEach((phrase) => {
            const words = phrase.split(" ");
            let currentRow = "";
            words.forEach((word) => {
                const attempt = currentRow ? `${currentRow} ${word}` : word;
                if (currentRow && ctx.measureText(attempt).width > wrapWidth) {
                    rows.push(currentRow);
                    currentRow = word;
                } else {
                    currentRow = attempt;
                }
            });
            if (currentRow) rows.push(currentRow);
        });

        rows.forEach((row, i) => {
            const y = lineHeight * (i + 1) - (lineHeight - fontSize) / 2 - 2;
            ctx.fillText(row, 0, y);
        });
        ctx.setTransform(1, 0, 0, 1, 0, 0);

        gl.viewport(0, 0, this.canvas.width, this.canvas.height);
        gl.bindTexture(gl.TEXTURE_2D, this.texture);
        gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, true);
        gl.texImage2D(
            gl.TEXTURE_2D,
            0,
            gl.RGBA,
            gl.RGBA,
            gl.UNSIGNED_BYTE,
            this.textCanvas,
        );
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

        gl.uniform2f(
            this.uniforms.uResolution,
            this.canvas.width,
            this.canvas.height,
        );
        gl.uniform1f(this.uniforms.uBlockSize, 8 * this.dpr);

        this.render();
    }

    private render() {
        const { gl } = this;

        // Two-stage easing, not a single raw lerp: the cursor position is
        // smoothed first, then its frame-to-frame delta (velocity) is
        // smoothed again before it ever reaches the shader. A lone
        // position-lerp still snaps the *velocity* around every frame;
        // easing velocity too is what actually reads as a graceful,
        // cubic-feeling glide instead of a twitchy follow.
        const positionEase = 0.09;
        this.mouse.x += (this.targetMouse.x - this.mouse.x) * positionEase;
        this.mouse.y += (this.targetMouse.y - this.mouse.y) * positionEase;

        let rawVelX = (this.mouse.x - this.prevMouse.x) * this.canvas.width;
        let rawVelY = (this.mouse.y - this.prevMouse.y) * this.canvas.height;
        this.prevMouse.x = this.mouse.x;
        this.prevMouse.y = this.mouse.y;

        const maxVel = 50 * this.dpr;
        const rawVelLen = Math.hypot(rawVelX, rawVelY);
        if (rawVelLen > maxVel) {
            rawVelX = (rawVelX / rawVelLen) * maxVel;
            rawVelY = (rawVelY / rawVelLen) * maxVel;
        }

        const velocityEase = 0.14;
        this.velocity.x += (rawVelX - this.velocity.x) * velocityEase;
        this.velocity.y += (rawVelY - this.velocity.y) * velocityEase;
        const velX = this.velocity.x;
        const velY = this.velocity.y;

        gl.useProgram(this.program);
        gl.clearColor(0, 0, 0, 0);
        gl.clear(gl.COLOR_BUFFER_BIT);
        gl.enable(gl.BLEND);
        gl.blendFunc(gl.SRC_ALPHA, gl.ONE_MINUS_SRC_ALPHA);

        gl.uniform2f(this.uniforms.uMouse, this.mouse.x, this.mouse.y);
        gl.uniform2f(this.uniforms.uVelocityPx, velX, velY);
        gl.uniform1f(
            this.uniforms.uTime,
            (performance.now() - this.startTime) / 1000,
        );

        gl.drawArrays(gl.TRIANGLES, 0, 3);
    }

    private loop = () => {
        this.rafId = requestAnimationFrame(this.loop);
        this.render();
    };

    public stop() {
        if (this.rafId) cancelAnimationFrame(this.rafId);
        this.rafId = null;
        this.themeObserver?.disconnect();
        this.themeObserver = null;
        this.gl.deleteTexture(this.texture);
        this.gl.deleteProgram(this.program);
        // Restore the plain, always-accessible heading on cleanup.
        this.el.style.color = "";
    }
}
