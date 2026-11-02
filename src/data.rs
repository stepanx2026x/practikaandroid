//! Игровые данные: героини, наряды, навыки, талисманы, локации и ёкаи.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hair {
    Long,
    Twin,
    Pony,
    Bob,
    Buns,
    Hime,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weapon {
    Katana,
    Ofuda,
    Bow,
    Kunai,
    Naginata,
    Fan,
    Lantern,
    Staff,
}

/// Аксессуары и детали образа.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Extras {
    pub fox: bool,
    pub kabuto: bool,
    pub hood: bool,
    pub halo: bool,
    pub flower: bool,
    pub scarf: bool,
    pub ribbon: bool,
    pub headband: bool,
    pub pattern: bool,
}

impl Extras {
    pub const NONE: Extras = Extras {
        fox: false,
        kabuto: false,
        hood: false,
        halo: false,
        flower: false,
        scarf: false,
        ribbon: false,
        headband: false,
        pattern: false,
    };
}

/// Внешний вид героини.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Look {
    pub hair: u32,
    pub style: Hair,
    pub eyes: u32,
    pub top: u32,
    pub lower: u32,
    pub obi: u32,
    pub weapon: Weapon,
    pub ex: Extras,
}

pub struct HeroineDef {
    pub id: &'static str,
    pub name: &'static str,
    pub title: &'static str,
    pub clicker: bool,
    pub base_cost: f64,
    pub base_dps: f64,
    pub look: Look,
}

const fn look(
    hair: u32,
    style: Hair,
    eyes: u32,
    top: u32,
    lower: u32,
    obi: u32,
    weapon: Weapon,
    ex: Extras,
) -> Look {
    Look {
        hair,
        style,
        eyes,
        top,
        lower,
        obi,
        weapon,
        ex,
    }
}

const fn hero(
    id: &'static str,
    name: &'static str,
    title: &'static str,
    cost: f64,
    dps: f64,
    look: Look,
) -> HeroineDef {
    HeroineDef {
        id,
        name,
        title,
        clicker: false,
        base_cost: cost,
        base_dps: dps,
        look,
    }
}

pub const SAKURA_HAIR: u32 = 0xF58DB0;
pub const SAKURA_EYES: u32 = 0xD8336F;

pub static HEROINES: [HeroineDef; 14] = [
    HeroineDef {
        id: "sakura",
        name: "Сакура",
        title: "главная героиня • ваш удар",
        clicker: true,
        base_cost: 5.0,
        base_dps: 0.0,
        look: look(
            SAKURA_HAIR,
            Hair::Long,
            SAKURA_EYES,
            0xFAD1DF,
            0xF48FB1,
            0xC2185B,
            Weapon::Katana,
            Extras {
                flower: true,
                pattern: true,
                ..Extras::NONE
            },
        ),
    },
    hero(
        "hana",
        "Хана",
        "юная мечница",
        50.0,
        5.0,
        look(
            0x8D5A3B,
            Hair::Pony,
            0xC77D2E,
            0xB9E4B0,
            0x4C9A5B,
            0xF6D365,
            Weapon::Katana,
            Extras {
                headband: true,
                ..Extras::NONE
            },
        ),
    ),
    hero(
        "rin",
        "Рин",
        "мико с талисманами",
        250.0,
        22.0,
        look(
            0x2B2B3A,
            Hair::Hime,
            0xB23A48,
            0xFFFFFF,
            0xD93A3A,
            0xD93A3A,
            Weapon::Ofuda,
            Extras {
                ribbon: true,
                ..Extras::NONE
            },
        ),
    ),
    hero(
        "yuki",
        "Юки",
        "лучница горных рек",
        1_000.0,
        74.0,
        look(
            0x9AD3F0,
            Hair::Bob,
            0x3A8FD6,
            0xE3F4FF,
            0x4AA3DF,
            0x1F5F9E,
            Weapon::Bow,
            Extras::NONE,
        ),
    ),
    hero(
        "kage",
        "Кагэ",
        "тень-ниндзя",
        4_000.0,
        245.0,
        look(
            0x3B4252,
            Hair::Pony,
            0x8E6CD8,
            0x4A5468,
            0x2E3440,
            0xE0457B,
            Weapon::Kunai,
            Extras {
                scarf: true,
                ..Extras::NONE
            },
        ),
    ),
    hero(
        "ayame",
        "Аямэ",
        "мастер нагинаты",
        20_000.0,
        976.0,
        look(
            0x6A4BAF,
            Hair::Long,
            0x9B7BDF,
            0xE4DAF7,
            0x8E6FCF,
            0xF2C14E,
            Weapon::Naginata,
            Extras {
                pattern: true,
                ..Extras::NONE
            },
        ),
    ),
    hero(
        "tomoe",
        "Томоэ",
        "странствующий самурай",
        100_000.0,
        3_725.0,
        look(
            0x3E2723,
            Hair::Pony,
            0x8B4A2B,
            0xC62828,
            0x37474F,
            0xF2C14E,
            Weapon::Katana,
            Extras {
                kabuto: true,
                ..Extras::NONE
            },
        ),
    ),
    hero(
        "mia",
        "Мия",
        "лисья волшебница",
        400_000.0,
        10_860.0,
        look(
            0xFFA040,
            Hair::Long,
            0xE69A1E,
            0xFFF3E0,
            0xFF7A45,
            0xC2410C,
            Weapon::Fan,
            Extras {
                fox: true,
                ..Extras::NONE
            },
        ),
    ),
    hero(
        "akari",
        "Акари",
        "хранительница фонарей",
        2.5e6,
        47_150.0,
        look(
            0xFF7F5C,
            Hair::Buns,
            0xE8652B,
            0xFFE2C6,
            0xFF8A65,
            0xB83A1C,
            Weapon::Lantern,
            Extras {
                pattern: true,
                ..Extras::NONE
            },
        ),
    ),
    hero(
        "shizuka",
        "Сидзука",
        "мастер чайной церемонии",
        1.5e7,
        186_000.0,
        look(
            0x2F5D3A,
            Hair::Hime,
            0x3E8E5B,
            0xD5EED9,
            0x6FB07F,
            0x2E6B3F,
            Weapon::Fan,
            Extras {
                flower: true,
                ..Extras::NONE
            },
        ),
    ),
    hero(
        "ryuko",
        "Рюко",
        "укротительница драконов",
        1e8,
        782_000.0,
        look(
            0xB5243A,
            Hair::Twin,
            0xD62F45,
            0x3A3F4B,
            0x22252D,
            0xFF5470,
            Weapon::Naginata,
            Extras::NONE,
        ),
    ),
    hero(
        "tsukiko",
        "Цукико",
        "лунная жрица",
        8e8,
        3.72e6,
        look(
            0xDADDE8,
            Hair::Long,
            0x6B7FD7,
            0x3F51B5,
            0x283593,
            0xB3E5FC,
            Weapon::Staff,
            Extras {
                hood: true,
                ..Extras::NONE
            },
        ),
    ),
    hero(
        "amatera",
        "Аматэра",
        "солнечная императрица",
        6.5e9,
        1.44e7,
        look(
            0xFFCC40,
            Hair::Long,
            0xF29F05,
            0xFFF8D6,
            0xFFD36B,
            0xFF7A00,
            Weapon::Staff,
            Extras {
                halo: true,
                pattern: true,
                ..Extras::NONE
            },
        ),
    ),
    hero(
        "hotaru",
        "Хотару",
        "повелительница светлячков",
        6e10,
        6.5e7,
        look(
            0xA6D96A,
            Hair::Twin,
            0x6DBE45,
            0x263A6B,
            0x1B2A50,
            0xC6FF00,
            Weapon::Lantern,
            Extras::NONE,
        ),
    ),
];

/// Наряд Сакуры. Выпадает с босса этапа `unlock_stage`.
pub struct OutfitDef {
    pub id: &'static str,
    pub name: &'static str,
    pub unlock_stage: u32,
    pub bonus: &'static str,
    pub top: u32,
    pub lower: u32,
    pub obi: u32,
    pub ex: Extras,
    pub click_mult: f64,
    pub dps_mult: f64,
    pub gold_mult: f64,
    pub crit_add: f64,
    pub timer_add: f64,
}

const fn outfit(
    id: &'static str,
    name: &'static str,
    unlock_stage: u32,
    bonus: &'static str,
    top: u32,
    lower: u32,
    obi: u32,
    ex: Extras,
) -> OutfitDef {
    OutfitDef {
        id,
        name,
        unlock_stage,
        bonus,
        top,
        lower,
        obi,
        ex,
        click_mult: 1.0,
        dps_mult: 1.0,
        gold_mult: 1.0,
        crit_add: 0.0,
        timer_add: 0.0,
    }
}

pub static OUTFITS: [OutfitDef; 9] = [
    outfit(
        "casual",
        "Повседневное кимоно",
        0,
        "Без бонусов",
        0xFAD1DF,
        0xF48FB1,
        0xC2185B,
        Extras {
            flower: true,
            pattern: true,
            ..Extras::NONE
        },
    ),
    OutfitDef {
        gold_mult: 1.25,
        ..outfit(
            "yukata",
            "Юката фестиваля",
            5,
            "Золото +25%",
            0x7CCBF5,
            0x3BA6E8,
            0xFFD54F,
            Extras {
                flower: true,
                pattern: true,
                ..Extras::NONE
            },
        )
    },
    OutfitDef {
        click_mult: 1.5,
        ..outfit(
            "miko",
            "Наряд мико",
            10,
            "Удар ×1,5",
            0xFFFFFF,
            0xE53935,
            0xE53935,
            Extras {
                ribbon: true,
                flower: true,
                ..Extras::NONE
            },
        )
    },
    OutfitDef {
        click_mult: 1.25,
        dps_mult: 1.25,
        ..outfit(
            "samurai",
            "Доспехи самурая",
            20,
            "Удар ×1,25 • отряд ×1,25",
            0xB71C1C,
            0x37474F,
            0x212121,
            Extras {
                kabuto: true,
                ..Extras::NONE
            },
        )
    },
    OutfitDef {
        crit_add: 0.07,
        ..outfit(
            "ninja",
            "Одежда ниндзя",
            30,
            "Шанс крита +7%",
            0x434B5C,
            0x263238,
            0xE91E63,
            Extras {
                scarf: true,
                flower: true,
                ..Extras::NONE
            },
        )
    },
    OutfitDef {
        timer_add: 7.0,
        ..outfit(
            "winter",
            "Зимнее кимоно",
            40,
            "Время на босса +7 с",
            0xE3F2FD,
            0x90CAF9,
            0x1E88E5,
            Extras {
                hood: true,
                pattern: true,
                ..Extras::NONE
            },
        )
    },
    OutfitDef {
        gold_mult: 1.75,
        ..outfit(
            "kitsune",
            "Наряд лисьей жрицы",
            50,
            "Золото ×1,75",
            0xFFF3E0,
            0xFF7043,
            0xBF360C,
            Extras {
                fox: true,
                ..Extras::NONE
            },
        )
    },
    OutfitDef {
        click_mult: 1.5,
        dps_mult: 1.5,
        ..outfit(
            "furisode",
            "Церемониальное фурисодэ",
            75,
            "Удар ×1,5 • отряд ×1,5",
            0x9C3BC0,
            0x6A1B9A,
            0xFFD54F,
            Extras {
                flower: true,
                pattern: true,
                ..Extras::NONE
            },
        )
    },
    OutfitDef {
        click_mult: 2.0,
        dps_mult: 2.0,
        gold_mult: 1.5,
        ..outfit(
            "goddess",
            "Облачение богини сакуры",
            100,
            "Всё ×2 • золото ×1,5",
            0xFFFFFF,
            0xF8BBD0,
            0xFFC107,
            Extras {
                halo: true,
                flower: true,
                pattern: true,
                ..Extras::NONE
            },
        )
    },
];

/// Образ Сакуры в наряде.
pub fn sakura_look(outfit: &OutfitDef) -> Look {
    Look {
        hair: SAKURA_HAIR,
        style: Hair::Long,
        eyes: SAKURA_EYES,
        top: outfit.top,
        lower: outfit.lower,
        obi: outfit.obi,
        weapon: Weapon::Katana,
        ex: outfit.ex,
    }
}

pub struct SkillDef {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub unlock_stage: u32,
    pub duration: f64,
    pub cooldown: f64,
}

pub static SKILLS: [SkillDef; 4] = [
    SkillDef {
        id: "storm",
        name: "Буря лепестков",
        desc: "10 ударов в секунду на 30 с",
        unlock_stage: 3,
        duration: 30.0,
        cooldown: 300.0,
    },
    SkillDef {
        id: "blessing",
        name: "Благословение",
        desc: "Урон отряда ×2 на 30 с",
        unlock_stage: 10,
        duration: 30.0,
        cooldown: 600.0,
    },
    SkillDef {
        id: "fan",
        name: "Золотой веер",
        desc: "Золото ×2 на 30 с",
        unlock_stage: 20,
        duration: 30.0,
        cooldown: 600.0,
    },
    SkillDef {
        id: "strike",
        name: "Небесный удар",
        desc: "Мгновенно: 60 с урона отряда",
        unlock_stage: 35,
        duration: 0.0,
        cooldown: 900.0,
    },
];
pub const SKILL_STORM: usize = 0;
pub const SKILL_BLESSING: usize = 1;
pub const SKILL_FAN: usize = 2;
pub const SKILL_STRIKE: usize = 3;

pub struct TalismanDef {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub base_cost: u64,
    pub max_level: u32,
}

pub static TALISMANS: [TalismanDef; 5] = [
    TalismanDef {
        id: "blade",
        name: "Талисман клинка",
        desc: "+25% к удару Сакуры за уровень",
        base_cost: 1,
        max_level: 50,
    },
    TalismanDef {
        id: "spirit",
        name: "Талисман духа",
        desc: "+15% к урону отряда за уровень",
        base_cost: 1,
        max_level: 50,
    },
    TalismanDef {
        id: "wealth",
        name: "Манэки-нэко",
        desc: "+10% золота за уровень",
        base_cost: 1,
        max_level: 50,
    },
    TalismanDef {
        id: "luck",
        name: "Омамори удачи",
        desc: "+2% шанс крита за уровень",
        base_cost: 2,
        max_level: 10,
    },
    TalismanDef {
        id: "hourglass",
        name: "Песочные часы",
        desc: "+2 с на битву с боссом за уровень",
        base_cost: 2,
        max_level: 10,
    },
];
pub const TAL_BLADE: usize = 0;
pub const TAL_SPIRIT: usize = 1;
pub const TAL_WEALTH: usize = 2;
pub const TAL_LUCK: usize = 3;
pub const TAL_HOURGLASS: usize = 4;

// ---------------------------------------------------------------------------
// Ёкаи
// ---------------------------------------------------------------------------

/// Шаблон рисунка ёкая.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Kappa,
    Tanuki,
    Umbrella,
    Oni,
    Cat,
    Weasel,
    Tengu,
    Wall,
    Ghost,
    Spider,
    Lantern,
    Faceless,
    OneEye,
    Nue,
    SnowLady,
    SnowChild,
    Icicle,
    Fireball,
    Wheel,
    Raiju,
    Cloud,
    Dragon,
    Orochi,
}

pub struct MonsterDef {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: Kind,
    pub body: u32,
    pub accent: u32,
}

const fn m(id: &'static str, name: &'static str, kind: Kind, body: u32, accent: u32) -> MonsterDef {
    MonsterDef {
        id,
        name,
        kind,
        body,
        accent,
    }
}

pub static MONSTERS: [MonsterDef; 28] = [
    // 0..3 — Сакуровая роща
    m("kappa", "Каппа", Kind::Kappa, 0x6CC070, 0xF5D76E),
    m("tanuki", "Тануки", Kind::Tanuki, 0xA67C62, 0x5B4033),
    m("kasa", "Каса-обакэ", Kind::Umbrella, 0x9575CD, 0xF06292),
    m("red_oni", "Красный они", Kind::Oni, 0xE8453C, 0xF2C14E),
    // 4..7 — Бамбуковый лес
    m("bakeneko", "Бакэнэко", Kind::Cat, 0x3C3F4A, 0xFFD54F),
    m("kamaitachi", "Кама-итати", Kind::Weasel, 0xE9D7C0, 0xA1887F),
    m("karasu", "Карасу-тэнгу", Kind::Tengu, 0x37474F, 0xFFB300),
    m("tengu", "Великий тэнгу", Kind::Tengu, 0xD84335, 0xFFFFFF),
    // 8..11 — Горный храм
    m("nurikabe", "Нурикабэ", Kind::Wall, 0xA7A39E, 0x7CB342),
    m("monk", "Ёкай-монах", Kind::Ghost, 0xB8D4E8, 0x8D6E63),
    m("stone_oni", "Каменный они", Kind::Oni, 0x8C9AA6, 0x6D7B86),
    m("tsuchigumo", "Цутигумо", Kind::Spider, 0x5D4037, 0xFF5252),
    // 12..15 — Город фонарей
    m("chochin", "Тётин-обакэ", Kind::Lantern, 0xFF8A50, 0xD84315),
    m("noppera", "Ноппэрапон", Kind::Faceless, 0xFFF1E6, 0x7E57C2),
    m(
        "hitotsume",
        "Хитоцумэ-кодзо",
        Kind::OneEye,
        0xFFD7B5,
        0x5C6BC0,
    ),
    m("nue", "Нуэ", Kind::Nue, 0xE0A040, 0x5D4037),
    // 16..19 — Снежный перевал
    m("yukionna", "Юки-онна", Kind::SnowLady, 0xEAF6FF, 0x2B3A55),
    m("yukinko", "Юкинко", Kind::SnowChild, 0xFFFFFF, 0xC79A5B),
    m("tsurara", "Цурара-кодзо", Kind::Icicle, 0xA8E0FF, 0x4FC3F7),
    m("ice_oni", "Ледяной они", Kind::Oni, 0x5BB8F0, 0xE1F5FE),
    // 20..23 — Огненный вулкан
    m("hinotama", "Хи-но-тама", Kind::Fireball, 0xFF7043, 0xFFEB3B),
    m("fire_oni", "Огненный они", Kind::Oni, 0xFF6D2E, 0xFFD54F),
    m("kaen", "Каэн-обакэ", Kind::Ghost, 0xFF9E6B, 0xFFD54F),
    m("wanyudo", "Ванъюдо", Kind::Wheel, 0x6D4C41, 0xFF6F00),
    // 24..27 — Небесный дворец
    m("raiju", "Райдзю", Kind::Raiju, 0xFFE066, 0x4FC3F7),
    m("cloud", "Облачный дух", Kind::Cloud, 0xFFFFFF, 0x90CAF9),
    m(
        "dragonet",
        "Золотой дракончик",
        Kind::Dragon,
        0xFFC93C,
        0xFF7043,
    ),
    m("orochi", "Ямата-но Ороти", Kind::Orochi, 0x7CB342, 0xF2E28B),
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Weather {
    Petals,
    Leaves,
    Fireflies,
    Snow,
    Embers,
    Sparkles,
}

pub struct ZoneDef {
    pub name: &'static str,
    pub monsters: [usize; 3],
    pub boss: usize,
    pub weather: Weather,
    pub night: bool,
}

pub static ZONES: [ZoneDef; 7] = [
    ZoneDef {
        name: "Сакуровая роща",
        monsters: [0, 1, 2],
        boss: 3,
        weather: Weather::Petals,
        night: false,
    },
    ZoneDef {
        name: "Бамбуковый лес",
        monsters: [4, 5, 6],
        boss: 7,
        weather: Weather::Leaves,
        night: false,
    },
    ZoneDef {
        name: "Горный храм",
        monsters: [8, 9, 10],
        boss: 11,
        weather: Weather::Petals,
        night: false,
    },
    ZoneDef {
        name: "Город фонарей",
        monsters: [12, 13, 14],
        boss: 15,
        weather: Weather::Fireflies,
        night: true,
    },
    ZoneDef {
        name: "Снежный перевал",
        monsters: [16, 17, 18],
        boss: 19,
        weather: Weather::Snow,
        night: false,
    },
    ZoneDef {
        name: "Огненный вулкан",
        monsters: [20, 21, 22],
        boss: 23,
        weather: Weather::Embers,
        night: false,
    },
    ZoneDef {
        name: "Небесный дворец",
        monsters: [24, 25, 26],
        boss: 27,
        weather: Weather::Sparkles,
        night: false,
    },
];

pub fn zone_index(stage: u32) -> usize {
    ((stage.max(1) - 1) / 10) as usize % ZONES.len()
}

/// Название локации с номером круга (после всех локаций они повторяются сильнее).
pub fn zone_name(stage: u32) -> String {
    let round = (stage.max(1) - 1) / 10 / ZONES.len() as u32;
    let base = ZONES[zone_index(stage)].name;
    const ROMAN: [&str; 11] = [
        "", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X",
    ];
    if round == 0 {
        base.to_string()
    } else if ((round + 1) as usize) < ROMAN.len() {
        format!("{} {}", base, ROMAN[(round + 1) as usize])
    } else {
        format!("{} {}", base, round + 1)
    }
}
