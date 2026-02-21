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
            icon: "M5 12h14M12 5l7 7-7 7", // Minimal right arrow
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
        icon: "M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z", // Clean moon
        action: () => {
            const isDark = document.documentElement.classList.toggle("dark");
            localStorage.setItem("theme", isDark ? "dark" : "light");
        },
    });

    commands.push({
        id: "system-scroll-top",
        title: "Scroll to Top",
        section: "System",
        icon: "M12 19V5M5 12l7-7 7 7", // Minimal up arrow
        action: () => {
            window.scrollTo({ top: 0, behavior: "smooth" });
        },
    });

    commands.push({
        id: "system-close-menu",
        title: "Close menu",
        section: "System",
        shortcut: ["Esc"],
        icon: "M18 6L6 18M6 6l12 12", // Clean X
        action: () => {
            document.dispatchEvent(new CustomEvent("cmdk-close"));
        },
    });

    return commands;
};
