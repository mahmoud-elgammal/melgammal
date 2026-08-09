/**
 * Keyboard Shortcut Configuration
 *
 * Uses GitHub-style "go to" pattern: press `g` then a letter key.
 * Example: `g h` → navigate to Home
 *
 * Design pattern: Config-driven — all shortcuts defined declaratively,
 * registered in one place via tinykeys.
 */

export interface ShortcutEntry {
    /** tinykeys key sequence, e.g. "g h" */
    keys: string;
    /** Navigation target URL */
    href: string;
    /** Human-readable label for the shortcut */
    label: string;
    /** Display key hint, e.g. "G H" */
    hint: string;
    /** Split keys for UI display, e.g. ["G", "H"] */
    displayKeys: string[];
}

export const NAV_SHORTCUTS: ShortcutEntry[] = [
    { keys: "g h", href: "/", label: "Home", hint: "G H", displayKeys: ["G", "H"] },
    { keys: "g w", href: "/work", label: "Works", hint: "G W", displayKeys: ["G", "W"] },
    { keys: "g p", href: "/playground", label: "Playground", hint: "G P", displayKeys: ["G", "P"] },
    { keys: "g r", href: "/process", label: "Process", hint: "G R", displayKeys: ["G", "R"] },
    { keys: "g a", href: "/about", label: "About", hint: "G A", displayKeys: ["G", "A"] },
    { keys: "g c", href: "/contact", label: "Contact", hint: "G C", displayKeys: ["G", "C"] },
    { keys: "g b", href: "/blog", label: "Blog", hint: "G B", displayKeys: ["G", "B"] },
    { keys: "g v", href: "/resume", label: "Resume", hint: "G V", displayKeys: ["G", "V"] },
];
