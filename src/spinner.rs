/// A spinner animation definition
#[derive(Debug, Clone)]
pub struct SpinnerDef {
    pub frames: &'static [&'static str],
    pub interval_ms: u64,
}

// Define all 44 spinners as static constants
const DOTS: SpinnerDef = SpinnerDef {
    frames: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
    interval_ms: 80,
};

const DOTS2: SpinnerDef = SpinnerDef {
    frames: &["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"],
    interval_ms: 80,
};

const DOTS3: SpinnerDef = SpinnerDef {
    frames: &["⠋", "⠙", "⠚", "⠞", "⠖", "⠦", "⠴", "⠲", "⠳", "⠓"],
    interval_ms: 80,
};

const DOTS4: SpinnerDef = SpinnerDef {
    frames: &["⠄", "⠆", "⠇", "⠋", "⠙", "⠸", "⠰", "⠠", "⠐", "⠈"],
    interval_ms: 80,
};

const LINE: SpinnerDef = SpinnerDef {
    frames: &["-", "\\", "|", "/"],
    interval_ms: 130,
};

const PIPE: SpinnerDef = SpinnerDef {
    frames: &["┤", "┘", "┴", "└", "├", "┌", "┬", "┐"],
    interval_ms: 100,
};

const SIMPLE_DOTS: SpinnerDef = SpinnerDef {
    frames: &[".  ", ".. ", "...", "   "],
    interval_ms: 400,
};

const STAR: SpinnerDef = SpinnerDef {
    frames: &["✶", "✸", "✹", "✺", "✹", "✸"],
    interval_ms: 100,
};

const SPARK: SpinnerDef = SpinnerDef {
    frames: &["·", "✦", "✧", "✦"],
    interval_ms: 150,
};

const ARC: SpinnerDef = SpinnerDef {
    frames: &["◜", "◠", "◝", "◞", "◡", "◟"],
    interval_ms: 100,
};

const CIRCLE: SpinnerDef = SpinnerDef {
    frames: &["◐", "◓", "◑", "◒"],
    interval_ms: 120,
};

const SQUARE_SPIN: SpinnerDef = SpinnerDef {
    frames: &["◰", "◳", "◲", "◱"],
    interval_ms: 120,
};

const TRIANGLES: SpinnerDef = SpinnerDef {
    frames: &["◢", "◣", "◤", "◥"],
    interval_ms: 120,
};

const SECTORS: SpinnerDef = SpinnerDef {
    frames: &["◴", "◷", "◶", "◵"],
    interval_ms: 120,
};

const DIAMOND: SpinnerDef = SpinnerDef {
    frames: &["◇", "◈", "◆", "◈"],
    interval_ms: 200,
};

const TOGGLE: SpinnerDef = SpinnerDef {
    frames: &["▪", "▫"],
    interval_ms: 300,
};

const TOGGLE2: SpinnerDef = SpinnerDef {
    frames: &["◼", "◻"],
    interval_ms: 300,
};

const BLOCKS: SpinnerDef = SpinnerDef {
    frames: &["░", "▒", "▓", "█", "▓", "▒"],
    interval_ms: 100,
};

const BLOCKS2: SpinnerDef = SpinnerDef {
    frames: &["▖", "▘", "▝", "▗"],
    interval_ms: 100,
};

const BLOCKS3: SpinnerDef = SpinnerDef {
    frames: &["▌", "▀", "▐", "▄"],
    interval_ms: 100,
};

const PULSE: SpinnerDef = SpinnerDef {
    frames: &["·", "•", "●", "•"],
    interval_ms: 150,
};

const PULSE2: SpinnerDef = SpinnerDef {
    frames: &["○", "◎", "●", "◎"],
    interval_ms: 150,
};

const BREATHE: SpinnerDef = SpinnerDef {
    frames: &["  ∙  ", " ∙∙∙ ", "∙∙∙∙∙", " ∙∙∙ "],
    interval_ms: 200,
};

const HEARTBEAT: SpinnerDef = SpinnerDef {
    frames: &["♡", "♡", "♥", "♥", "♡", "♡", " ", " "],
    interval_ms: 150,
};

const GROWING: SpinnerDef = SpinnerDef {
    frames: &[
        "▏", "▎", "▍", "▌", "▋", "▊", "▉", "█", "▉", "▊", "▋", "▌", "▍", "▎",
    ],
    interval_ms: 80,
};

const BOUNCE: SpinnerDef = SpinnerDef {
    frames: &["⠁", "⠂", "⠄", "⡀", "⢀", "⠠", "⠐", "⠈"],
    interval_ms: 120,
};

const BOUNCING_BAR: SpinnerDef = SpinnerDef {
    frames: &[
        "[    =     ]",
        "[   =      ]",
        "[  =       ]",
        "[ =        ]",
        "[=         ]",
        "[ =        ]",
        "[  =       ]",
        "[   =      ]",
        "[    =     ]",
        "[     =    ]",
        "[      =   ]",
        "[       =  ]",
        "[        = ]",
        "[         =]",
        "[        = ]",
        "[       =  ]",
        "[      =   ]",
        "[     =    ]",
    ],
    interval_ms: 80,
};

const BOUNCING_BALL: SpinnerDef = SpinnerDef {
    frames: &[
        "( ●    )",
        "(  ●   )",
        "(   ●  )",
        "(    ● )",
        "(     ●)",
        "(    ● )",
        "(   ●  )",
        "(  ●   )",
        "( ●    )",
        "(●     )",
    ],
    interval_ms: 80,
};

const ARROWS: SpinnerDef = SpinnerDef {
    frames: &["←", "↖", "↑", "↗", "→", "↘", "↓", "↙"],
    interval_ms: 120,
};

const ARROW_PULSE: SpinnerDef = SpinnerDef {
    frames: &["▹▹▹▹▹", "►▹▹▹▹", "▹►▹▹▹", "▹▹►▹▹", "▹▹▹►▹", "▹▹▹▹►"],
    interval_ms: 120,
};

const WAVE: SpinnerDef = SpinnerDef {
    frames: &[
        "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█", "▇", "▆", "▅", "▄", "▃", "▂",
    ],
    interval_ms: 80,
};

const WAVE2: SpinnerDef = SpinnerDef {
    frames: &[
        "▁▂▃",
        "▂▃▄",
        "▃▄▅",
        "▄▅▆",
        "▅▆▇",
        "▆▇█",
        "▇█▇",
        "█▇▆",
        "▇▆▅",
        "▆▅▄",
        "▅▄▃",
        "▄▃▂",
        "▃▂▁",
    ],
    interval_ms: 80,
};

const AESTHETIC: SpinnerDef = SpinnerDef {
    frames: &[
        "▱▱▱▱▱",
        "▰▱▱▱▱",
        "▰▰▱▱▱",
        "▰▰▰▱▱",
        "▰▰▰▰▱",
        "▰▰▰▰▰",
        "▱▱▱▱▱",
    ],
    interval_ms: 150,
};

const FILLING: SpinnerDef = SpinnerDef {
    frames: &[
        "□□□□□",
        "■□□□□",
        "■■□□□",
        "■■■□□",
        "■■■■□",
        "■■■■■",
        "□□□□□",
    ],
    interval_ms: 150,
};

const SCANNING: SpinnerDef = SpinnerDef {
    frames: &[
        "░░░░░",
        "▒░░░░",
        "░▒░░░",
        "░░▒░░",
        "░░░▒░",
        "░░░░▒",
        "░░░░░",
    ],
    interval_ms: 100,
};

const BINARY: SpinnerDef = SpinnerDef {
    frames: &["010010", "001101", "100110", "110011", "011001", "101100"],
    interval_ms: 100,
};

const MATRIX: SpinnerDef = SpinnerDef {
    frames: &["Ξ", "Σ", "Φ", "Ψ", "Ω", "λ", "μ", "π"],
    interval_ms: 100,
};

const HACK: SpinnerDef = SpinnerDef {
    frames: &["▓▒░", "▒░▓", "░▓▒"],
    interval_ms: 100,
};

const BRAILLE_SNAKE: SpinnerDef = SpinnerDef {
    frames: &["⠏", "⠛", "⠹", "⢸", "⣰", "⣤", "⣆", "⡇"],
    interval_ms: 100,
};

const BRAILLE_WAVE: SpinnerDef = SpinnerDef {
    frames: &[
        "⠁", "⠂", "⠄", "⡀", "⡈", "⡐", "⡠", "⣀", "⣁", "⣂", "⣄", "⣌", "⣔", "⣤", "⣥", "⣦", "⣮", "⣶",
        "⣷", "⣿", "⡿", "⠿", "⢟", "⠟", "⠏", "⠇", "⠃", "⠁",
    ],
    interval_ms: 60,
};

const ORBIT: SpinnerDef = SpinnerDef {
    frames: &["◯", "◎", "●", "◎"],
    interval_ms: 200,
};

const EARTH: SpinnerDef = SpinnerDef {
    frames: &["🌍", "🌎", "🌏"],
    interval_ms: 300,
};

const MOON: SpinnerDef = SpinnerDef {
    frames: &["🌑", "🌒", "🌓", "🌔", "🌕", "🌖", "🌗", "🌘"],
    interval_ms: 200,
};

const CLOCK: SpinnerDef = SpinnerDef {
    frames: &[
        "🕐", "🕑", "🕒", "🕓", "🕔", "🕕", "🕖", "🕗", "🕘", "🕙", "🕚", "🕛",
    ],
    interval_ms: 150,
};

const HOURGLASS: SpinnerDef = SpinnerDef {
    frames: &["⏳", "⌛"],
    interval_ms: 500,
};

/// Get a spinner by name
pub fn get_spinner(name: &str) -> Option<&'static SpinnerDef> {
    match name {
        "dots" => Some(&DOTS),
        "dots2" => Some(&DOTS2),
        "dots3" => Some(&DOTS3),
        "dots4" => Some(&DOTS4),
        "line" => Some(&LINE),
        "pipe" => Some(&PIPE),
        "simpleDots" => Some(&SIMPLE_DOTS),
        "star" => Some(&STAR),
        "spark" => Some(&SPARK),
        "arc" => Some(&ARC),
        "circle" => Some(&CIRCLE),
        "squareSpin" => Some(&SQUARE_SPIN),
        "triangles" => Some(&TRIANGLES),
        "sectors" => Some(&SECTORS),
        "diamond" => Some(&DIAMOND),
        "toggle" => Some(&TOGGLE),
        "toggle2" => Some(&TOGGLE2),
        "blocks" => Some(&BLOCKS),
        "blocks2" => Some(&BLOCKS2),
        "blocks3" => Some(&BLOCKS3),
        "pulse" => Some(&PULSE),
        "pulse2" => Some(&PULSE2),
        "breathe" => Some(&BREATHE),
        "heartbeat" => Some(&HEARTBEAT),
        "growing" => Some(&GROWING),
        "bounce" => Some(&BOUNCE),
        "bouncingBar" => Some(&BOUNCING_BAR),
        "bouncingBall" => Some(&BOUNCING_BALL),
        "arrows" => Some(&ARROWS),
        "arrowPulse" => Some(&ARROW_PULSE),
        "wave" => Some(&WAVE),
        "wave2" => Some(&WAVE2),
        "aesthetic" => Some(&AESTHETIC),
        "filling" => Some(&FILLING),
        "scanning" => Some(&SCANNING),
        "binary" => Some(&BINARY),
        "matrix" => Some(&MATRIX),
        "hack" => Some(&HACK),
        "brailleSnake" => Some(&BRAILLE_SNAKE),
        "brailleWave" => Some(&BRAILLE_WAVE),
        "orbit" => Some(&ORBIT),
        "earth" => Some(&EARTH),
        "moon" => Some(&MOON),
        "clock" => Some(&CLOCK),
        "hourglass" => Some(&HOURGLASS),
        _ => None,
    }
}

/// List all spinner names
pub fn all_spinner_names() -> Vec<&'static str> {
    vec![
        "dots",
        "dots2",
        "dots3",
        "dots4",
        "line",
        "pipe",
        "simpleDots",
        "star",
        "spark",
        "arc",
        "circle",
        "squareSpin",
        "triangles",
        "sectors",
        "diamond",
        "toggle",
        "toggle2",
        "blocks",
        "blocks2",
        "blocks3",
        "pulse",
        "pulse2",
        "breathe",
        "heartbeat",
        "growing",
        "bounce",
        "bouncingBar",
        "bouncingBall",
        "arrows",
        "arrowPulse",
        "wave",
        "wave2",
        "aesthetic",
        "filling",
        "scanning",
        "binary",
        "matrix",
        "hack",
        "brailleSnake",
        "brailleWave",
        "orbit",
        "earth",
        "moon",
        "clock",
        "hourglass",
    ]
}
