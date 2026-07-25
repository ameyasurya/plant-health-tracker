//! One-time seed data for the 19 plants from Balcony_Plant_Inventory.md.
//!
//! This is loaded into plants.json on first run only (see store::ensure_seeded).
//! After that the user's own copy is the source of truth and this list is
//! never consulted again -- edits happen in-app, not by re-importing.
//!
//! Two plants are missing a field in the source inventory: Ixora has no
//! watering category and Indian Borage has no light category. Both are
//! filled with a reasonable default and marked `inferred: true` so the UI
//! can flag them for the user to confirm or correct.
//!
//! The uses/significance/fun_fact blurbs are general horticultural and
//! cultural background, written to be read in a small widget. Where a
//! plant has a traditional medicinal reputation it is described as a
//! tradition, not as health advice.

use crate::models::{default_space_id, FertilizeGroup, Light, MoistureClass, PlantProfile};

struct Knowledge {
    uses: &'static str,
    significance: &'static str,
    fun_fact: &'static str,
}

#[allow(clippy::too_many_arguments)]
fn p(
    id: &str,
    common_name: &str,
    scientific_name: &str,
    category: &str,
    light: Light,
    moisture_class: MoistureClass,
    fertilize_group: FertilizeGroup,
    is_hanging: bool,
    notes: &str,
    inferred: bool,
    knowledge: Knowledge,
) -> PlantProfile {
    PlantProfile {
        id: id.to_string(),
        common_name: common_name.to_string(),
        scientific_name: scientific_name.to_string(),
        category: category.to_string(),
        light,
        moisture_class,
        fertilize_group,
        is_hanging,
        notes: notes.to_string(),
        inferred,
        space_id: default_space_id(),
        uses: knowledge.uses.to_string(),
        significance: knowledge.significance.to_string(),
        fun_fact: knowledge.fun_fact.to_string(),
    }
}

fn k(uses: &'static str, significance: &'static str, fun_fact: &'static str) -> Knowledge {
    Knowledge { uses, significance, fun_fact }
}

pub fn seed_plants() -> Vec<PlantProfile> {
    use FertilizeGroup::*;
    use Light::*;
    use MoistureClass::*;

    vec![
        p("bougainvillea", "Bougainvillea", "Bougainvillea spp.", "Flowering", FullSun, Drier, FloweringFruiting, false, "Pink flowering climber; tall terracotta pot", false,
          k("Ornamental screening along railings and walls; thrives on heat and neglect once established.",
            "Named after the French navigator Louis Antoine de Bougainville, on whose 1760s voyage it was first recorded by Europeans in Brazil.",
            "The papery pink 'petals' are actually bracts -- the real flowers are the tiny white tubes tucked inside them.")),
        p("passion-fruit", "Passion Fruit", "Passiflora edulis", "Flowering/Fruiting", FullSun, ConsistentlyMoist, FloweringFruiting, false, "Young climber; wide terracotta pot", false,
          k("Edible fruit, plus a fast-growing vine for shade and screening on a balcony trellis.",
            "Spanish missionaries in South America read the flower's parts as symbols of the Passion of Christ, which is how it got its name.",
            "Each of those intricate flowers usually opens for just a single day.")),
        p("jasmine", "Jasmine", "Jasminum spp.", "Flowering", BrightLight, Moderate, FloweringFruiting, false, "Fragrant white flowers", false,
          k("Fragrant flowers used in garlands, hair adornment, perfumery and for scenting tea.",
            "Central to daily worship and celebration across South India, where fresh strings of mallige are sold by the metre.",
            "Many jasmines release their scent most strongly after dark, timed to night-flying pollinators.")),
        p("butterfly-pea", "Butterfly Pea", "Clitoria ternatea", "Flowering", FullSun, Moderate, FloweringFruiting, false, "Blue flowers", false,
          k("Flowers steeped into a vivid blue tea, and used as a natural food colouring.",
            "Known as Aparajita or Shankhpushpi in India and offered in worship; also long grown as a soil-enriching cover crop.",
            "Its blue tea turns pink the instant you add lemon -- the pigment is a natural pH indicator.")),
        p("purple-aster", "Purple Aster", "Callistephus / Symphyotrichum", "Flowering", BrightLight, Moderate, FloweringFruiting, false, "Lavender flowers", false,
          k("Ornamental colour and a valuable late-season nectar source for bees and butterflies.",
            "'Aster' is simply Greek for star, after the radiating shape of the flower head.",
            "What reads as one flower is really a composite head of dozens of tiny individual florets.")),
        p("ixora", "Ixora (Thetti)", "Ixora coccinea", "Flowering", FullSun, Moderate, FloweringFruiting, false, "Evergreen shrub. Watering class not in source inventory -- inferred from general Ixora care, please confirm.", true,
          k("Evergreen flowering shrub for hedging or containers; blooms in dense clusters much of the year.",
            "Called thetti in Kerala and widely used as a temple offering flower.",
            "Each ball of colour is a cluster of many slender, four-petalled tubular flowers.")),
        p("scented-geranium", "Scented Geranium", "Pelargonium spp.", "Fragrant", FullSun, Drier, HerbSucculent, false, "Lemon & mint fragrance", false,
          k("Aromatic leaves used to scent sugars, syrups and homemade soaps; grown for fragrance rather than showy bloom.",
            "Victorian gardeners collected dozens of scents -- rose, lemon, mint, nutmeg -- as prized parlour plants.",
            "The fragrance lives in the leaves, not the flowers, and only releases when you brush past or rub them.")),
        p("indian-borage", "Indian Borage", "Plectranthus amboinicus", "Medicinal", BrightLight, Drier, HerbSucculent, false, "Culinary & medicinal. Light class not in source inventory -- inferred as bright light, please confirm.", true,
          k("Thick aromatic leaves used sparingly as a culinary herb, and long featured in traditional home remedies across India.",
            "Goes by many regional names -- ajwain patta, karpooravalli, Mexican mint -- and is commonly kept just outside the kitchen door.",
            "Despite names like 'Mexican mint' and 'Cuban oregano', it originally comes from southern and eastern Africa.")),
        p("umbrella-tree", "Umbrella Tree", "Schefflera arboricola", "Tree", BrightLight, Drier, Foliage, false, "Structural foliage", false,
          k("Structural evergreen foliage for a shaded corner; takes pruning well and keeps a compact shape.",
            "One of the most widely grown foliage plants in the world, valued for tolerating low light and irregular care.",
            "Its leaflets radiate from a single point like the spokes of an umbrella -- hence the name.")),
        p("ficus", "Ficus", "Ficus spp.", "Shrub", BrightLight, Moderate, Foliage, false, "Large shrub", false,
          k("Large-leaved foliage shrub or small tree, good for screening and shade.",
            "The genus includes the sacred fig (peepal) and the banyan, among the most revered trees in India.",
            "Almost every fig species relies on its own specific species of tiny fig wasp to pollinate it.")),
        p("lemon-lime", "Lemon/Lime", "Citrus spp.", "Fruit Tree", FullSun, Moderate, Citrus, false, "Citrus", false,
          k("Fruit for the kitchen, plus fragrant blossom and evergreen aromatic leaves.",
            "Lemons are hung at doorways and used in rituals across India; citrus travelled west along ancient trade routes from Southeast Asia.",
            "Nearly all familiar citrus fruits are hybrids tracing back to just a few wild ancestors -- citron, pomelo and mandarin.")),
        p("orange", "Orange", "Citrus sp.", "Fruit Tree", FullSun, Moderate, Citrus, false, "Young tree", false,
          k("Edible fruit and intensely fragrant flowers.",
            "The sweet orange is thought to be an ancient hybrid first cultivated in China, reaching Europe through traders.",
            "An orange tree can carry flowers and ripening fruit at the very same time.")),
        p("curry-leaf", "Curry Leaf", "Murraya koenigii", "Herb/Tree", FullSun, Moderate, Foliage, false, "Culinary", false,
          k("Aromatic leaves used as a staple tempering ingredient in South Indian cooking.",
            "A near-essential kitchen-garden plant in Indian homes, where leaves are picked fresh straight off the plant.",
            "It belongs to the citrus family -- crush a leaf and you can catch a faint citrus note beneath the spice.")),
        p("bird-of-paradise", "Bird of Paradise", "Strelitzia nicolai (likely)", "Tropical", BrightIndirect, ConsistentlyMoist, Foliage, false, "Large foliage", false,
          k("Dramatic large-leaved foliage that anchors a bright corner.",
            "Named after Queen Charlotte of Mecklenburg-Strelitz; native to South Africa.",
            "Those huge leaves split along the veins on purpose -- it's wind adaptation, not damage.")),
        p("monstera-adansonii", "Monstera Adansonii", "Monstera adansonii", "Tropical Vine", BrightIndirect, Moderate, Foliage, false, "Swiss Cheese Vine", false,
          k("Trailing or climbing foliage vine for a bright indirect spot.",
            "Known as the Swiss Cheese Vine for its perforated leaves, and a mainstay of modern houseplant collections.",
            "The holes are called fenestrations, and they let wind and light pass through to the leaves below.")),
        p("golden-pothos", "Golden Pothos", "Epipremnum aureum", "Hanging", BrightIndirect, Moderate, Foliage, true, "Hanging vine", false,
          k("Hardy trailing vine for hanging pots and high shelves; tolerates low light well.",
            "Among the most forgiving houseplants there is -- often the first plant a new grower manages to keep alive.",
            "In the wild it climbs trees and its leaves grow enormous; indoors they stay small because it never gets to climb.")),
        p("english-ivy", "English Ivy", "Hedera helix", "Hanging", BrightIndirect, Moderate, Foliage, true, "Variegated", false,
          k("Trailing or climbing evergreen for hanging pots and screening.",
            "An old European symbol of fidelity and eternal life, wound into wreaths and crowns.",
            "Ivy leads two lives -- the familiar lobed climbing form, and a mature shrubby form with plain leaves that finally flowers.")),
        p("turtle-vine", "Turtle Vine", "Callisia repens", "Hanging", BrightIndirect, ConsistentlyMoist, HerbSucculent, true, "Trailing", false,
          k("Fast-spreading trailing groundcover, ideal for hanging baskets.",
            "A popular low-maintenance 'spiller' for the edges of mixed container plantings.",
            "The tiny overlapping leaves are said to resemble a turtle's shell -- hence the name.")),
        p("trailing-peperomia", "Trailing Peperomia", "Peperomia sp.", "Hanging", BrightIndirect, Drier, HerbSucculent, true, "Semi-succulent", false,
          k("Compact semi-succulent trailer for shelves and hanging pots.",
            "Peperomia is an enormous genus of over a thousand species, many growing on trees as epiphytes.",
            "Its thick leaves store water like a succulent, which is why overwatering harms it faster than forgetting to water.")),
    ]
}
