use desktop_sdk::Vfs;

/// Populates the filesystem the desktop boots with. This is the one place
/// that owns the site's actual content — apps just browse/read whatever's
/// here, the same way a real Finder/Terminal/Preview don't know in advance
/// what files exist on disk.
pub fn seed(vfs: &mut Vfs) {
    let _ = vfs.mkdir("/Home");
    let _ = vfs.write_file(
        "/Home/About.txt",
        "Mahmoud S. El-Gammal\n\
         Cross-Functional Developer\n\n\
         I work across design, engineering, and product, prioritizing \
         human understanding over just lines of code to bridge technology \
         with psychology and art.\n\n\
         Background in Psychology, driven by curiosity, logic, and craft. \
         Based in Egypt.",
    );
    let _ = vfs.write_file(
        "/Home/Resume.pdf",
        "Creative Technologist & System Architect\n\n\
         Node.js · TypeScript · React · Astro · Docker · Kubernetes · AWS\n\n\
         Full resume available at /resume",
    );
    let _ = vfs.write_file(
        "/Home/Contact.txt",
        "hello@melgammal.com\n\
         X · GitHub · LinkedIn\n\
         Cairo, Egypt",
    );

    let _ = vfs.mkdir("/Projects");
    for (name, stack) in [
        ("Syriazone", "Astro · Rust · PostgreSQL"),
        ("Crtly", "React Native · Node.js"),
        ("Tansto", "Framer · HealthTech"),
        ("Kleaner", "Astro · Tailwind"),
    ] {
        let dir = format!("/Projects/{name}");
        let _ = vfs.mkdir(&dir);
        let _ = vfs.write_file(&format!("{dir}/README.txt"), stack.to_string());
    }

    let _ = vfs.mkdir("/Blog");
    for (file, body) in [
        (
            "physics-engine.md",
            "Building a 2D Physics Engine from First Principles in TypeScript",
        ),
        (
            "kinetic-type.md",
            "Kinetic Typography: The Mathematics of Motion",
        ),
        (
            "digital-monoliths.md",
            "Architectural UX: Building Digital Monoliths",
        ),
        (
            "generative-systems.md",
            "Generative Systems: Coding Chaos at Scale",
        ),
    ] {
        let _ = vfs.write_file(&format!("/Blog/{file}"), body.to_string());
    }

    let _ = vfs.mkdir("/Playground");
    for (name, blurb) in [
        ("physics-sim", "A 2D physics playground."),
        ("shader-lab", "GLSL sketches and shader experiments."),
        ("kinetic-type", "Type that moves."),
    ] {
        let dir = format!("/Playground/{name}");
        let _ = vfs.mkdir(&dir);
        let _ = vfs.write_file(&format!("{dir}/README.txt"), blurb.to_string());
    }

    let _ = vfs.mkdir("/Trash");

    let _ = vfs.mkdir("/System");
    let _ = vfs.write_file(
        "/System/ReadMe.txt",
        "This desktop is a fake OS rendered on <canvas>, built with Rust, \
         egui, and WebAssembly. Every app you see here — Terminal, Finder, \
         Preview, and the rest — lives in its own crate behind a shared \
         DesktopApplet trait, wired together by a small shell crate that \
         owns the window manager, process table, and filesystem.",
    );
    let _ = vfs.write_file(
        "/System/Colophon.txt",
        "Renderer: glow (OpenGL/WebGL) via egui_glow.\n\
         Background: a hand-written GLSL shader, animated per frame.\n\
         Window chrome, traffic lights, and the dock are all hand-drawn \
         with egui's Painter — no images, no icon fonts.",
    );
    let _ = vfs.write_file(
        "/System/Release-Notes.txt",
        "v0.1 — Terminal, Finder, and Preview added as their own crates.\n\
         v0.1 — Wallpaper switched to an animated GLSL gradient.\n\
         v0.1 — Every app is a process: init boots WindowServer, which \
         spawns apps as they're opened.\n\
         v0.1 — Finder, Terminal, and Preview now share one real \
         filesystem instead of hardcoded lists.",
    );
}
