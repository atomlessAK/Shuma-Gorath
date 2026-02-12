use super::rng::SeededRng;

pub(super) const NOUNS: &[&str] = &[
    "system",
    "data",
    "server",
    "network",
    "client",
    "database",
    "file",
    "user",
    "admin",
    "config",
    "backup",
    "report",
    "dashboard",
    "analytics",
    "service",
    "process",
    "resource",
    "module",
    "component",
    "interface",
    "protocol",
    "session",
    "transaction",
    "record",
    "entry",
    "request",
    "response",
    "cache",
    "storage",
    "cluster",
    "node",
    "instance",
    "container",
    "deployment",
    "pipeline",
    "workflow",
];

pub(super) const VERBS: &[&str] = &[
    "configure",
    "manage",
    "update",
    "delete",
    "create",
    "view",
    "export",
    "import",
    "sync",
    "backup",
    "restore",
    "monitor",
    "analyze",
    "optimize",
    "validate",
    "process",
    "submit",
    "review",
    "approve",
    "deploy",
    "migrate",
    "transform",
];

pub(super) const ADJECTIVES: &[&str] = &[
    "advanced",
    "secure",
    "internal",
    "external",
    "primary",
    "secondary",
    "legacy",
    "updated",
    "archived",
    "active",
    "pending",
    "completed",
    "failed",
    "critical",
    "standard",
    "custom",
    "automated",
    "manual",
    "scheduled",
    "temporary",
    "permanent",
];

pub(super) const DEPARTMENTS: &[&str] = &[
    "Sales",
    "Marketing",
    "Engineering",
    "HR",
    "Finance",
    "Operations",
    "Support",
    "IT",
    "Legal",
    "Compliance",
    "Security",
    "Development",
    "QA",
    "DevOps",
];

const MONTHS: &[&str] = &[
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Capitalize first letter.
pub(super) fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Generate a fake title for a page.
pub(super) fn generate_title(rng: &mut SeededRng) -> String {
    let pattern = rng.next() % 4;
    match pattern {
        0 => {
            let adj = capitalize(rng.pick(ADJECTIVES));
            let noun = capitalize(rng.pick(NOUNS));
            format!("{} {} Management", adj, noun)
        }
        1 => {
            let dept = rng.pick(DEPARTMENTS);
            let noun = capitalize(rng.pick(NOUNS));
            format!("{} {} Portal", dept, noun)
        }
        2 => {
            let adj = capitalize(rng.pick(ADJECTIVES));
            let noun = capitalize(rng.pick(NOUNS));
            format!("{} {} Dashboard", adj, noun)
        }
        _ => {
            let verb = capitalize(rng.pick(VERBS));
            let noun = capitalize(rng.pick(NOUNS));
            let adj = capitalize(rng.pick(ADJECTIVES));
            format!("{} {} - {} Access", verb, noun, adj)
        }
    }
}

/// Generate a fake link text.
pub(super) fn generate_link_text(rng: &mut SeededRng) -> String {
    let pattern = rng.next() % 7;
    match pattern {
        0 => {
            let verb = capitalize(rng.pick(VERBS));
            let noun = capitalize(rng.pick(NOUNS));
            format!("{} {}", verb, noun)
        }
        1 => {
            let dept = rng.pick(DEPARTMENTS);
            let noun = rng.pick(NOUNS);
            format!("{} {} Portal", dept, noun)
        }
        2 => {
            let adj = capitalize(rng.pick(ADJECTIVES));
            let noun = capitalize(rng.pick(NOUNS));
            format!("{} {} Settings", adj, noun)
        }
        3 => {
            let adj = capitalize(rng.pick(ADJECTIVES));
            let noun = capitalize(rng.pick(NOUNS));
            format!("View {} {}", adj, noun)
        }
        4 => {
            let noun = capitalize(rng.pick(NOUNS));
            format!("{} Management", noun)
        }
        5 => {
            let dept = rng.pick(DEPARTMENTS);
            format!("{} Dashboard", dept)
        }
        _ => {
            let dept = rng.pick(DEPARTMENTS);
            let noun = rng.pick(NOUNS);
            format!("{} {} Report", dept, noun)
        }
    }
}

/// Generate a fake date string.
pub(super) fn generate_fake_date(rng: &mut SeededRng) -> String {
    let month = rng.pick(MONTHS);
    let day = rng.range(1, 28);
    let year_suffix = rng.range(3, 6);
    format!("{} {}, 202{}", month, day, year_suffix)
}

/// Generate a fake paragraph of text.
pub(super) fn generate_paragraph(rng: &mut SeededRng) -> String {
    let pattern = rng.next() % 5;
    match pattern {
        0 => {
            let adj1 = rng.pick(ADJECTIVES);
            let noun1 = rng.pick(NOUNS);
            let adj2 = rng.pick(ADJECTIVES);
            let adj3 = rng.pick(ADJECTIVES);
            let noun2 = rng.pick(NOUNS);
            let noun3 = rng.pick(NOUNS);
            format!(
                "The {} {} requires {} access to the {} {}. Please ensure all {} are properly configured before proceeding.",
                adj1, noun1, adj2, adj3, noun2, noun3
            )
        }
        1 => {
            let noun1 = rng.pick(NOUNS);
            let verb = rng.pick(VERBS);
            let adj = rng.pick(ADJECTIVES);
            let noun2 = rng.pick(NOUNS);
            let noun3 = rng.pick(NOUNS);
            format!(
                "This {} allows you to {} the {} {}. All changes are logged and can be reviewed in the {} section.",
                noun1, verb, adj, noun2, noun3
            )
        }
        2 => {
            let adj1 = rng.pick(ADJECTIVES);
            let noun = rng.pick(NOUNS);
            let adj2 = rng.pick(ADJECTIVES);
            let dept = rng.pick(DEPARTMENTS);
            format!(
                "Access to {} {} is restricted to {} personnel only. Contact {} for authorization requests.",
                adj1, noun, adj2, dept
            )
        }
        3 => {
            let noun1 = rng.pick(NOUNS);
            let noun2 = rng.pick(NOUNS);
            let date = generate_fake_date(rng);
            let noun3 = rng.pick(NOUNS);
            let noun4 = rng.pick(NOUNS);
            format!(
                "The {} {} was last updated on {}. Review the {} for recent changes and {}.",
                noun1, noun2, date, noun3, noun4
            )
        }
        _ => {
            let noun1 = rng.pick(NOUNS);
            let verb1 = rng.pick(VERBS);
            let noun2 = rng.pick(NOUNS);
            let noun3 = rng.pick(NOUNS);
            let noun4 = rng.pick(NOUNS);
            let verb2 = rng.pick(VERBS);
            format!(
                "Use this {} to {} {} across all {}. The {} will be {} automatically.",
                noun1, verb1, noun2, noun3, noun4, verb2
            )
        }
    }
}
