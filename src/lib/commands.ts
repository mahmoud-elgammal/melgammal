import { NAV_SHORTCUTS } from "./shortcuts";

export interface Command {
    id: string;
    title: string;
    description?: string;
    section: "Navigation" | "System" | "Social";
    icon?: string; // SVG path d-string
    action: () => void;
    shortcut?: string[]; // Display shortcut
}

export const getCommands = (): Command[] => {
    const commands: Command[] = [];

    // 1. Navigation Commands
    NAV_SHORTCUTS.forEach((s) => {
        commands.push({
            id: `nav-${s.label.toLowerCase()}`,
            title: `Go to ${s.label}`,
            section: "Navigation",
            shortcut: s.displayKeys,
            icon: "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6",
            action: () => {
                window.location.href = s.href;
            },
        });
    });

    // 2. System Commands
    commands.push({
        id: "system-theme-toggle",
        title: "Toggle Theme",
        description: "Switch between light and dark mode",
        section: "System",
        shortcut: ["T"],
        icon: "M12 3v1m0 16v1m9-9h-1M4 12H3m15.364-6.364l-.707.707M6.343 17.657l-.707.707m12.728 0l-.707-.707M6.343 6.343l-.707-.707M15 12a3 3 0 11-6 0 3 3 0 016 0z",
        action: () => {
            const isDark = document.documentElement.classList.toggle("dark");
            localStorage.setItem("theme", isDark ? "dark" : "light");
        },
    });

    commands.push({
        id: "system-scroll-top",
        title: "Scroll to Top",
        section: "System",
        icon: "M5 11l7-7 7 7M5 19l7-7 7 7",
        action: () => {
            window.scrollTo({ top: 0, behavior: "smooth" });
        },
    });

    commands.push({
        id: "system-close-menu",
        title: "Close menu",
        section: "System",
        shortcut: ["Esc"],
        icon: "M6 18L18 6M6 6l12 12",
        action: () => {
            document.dispatchEvent(new CustomEvent("cmdk-close"));
        },
    });

    return commands;
};
