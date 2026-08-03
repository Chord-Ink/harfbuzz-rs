//! Script identifiers, one per ISO 15924 script — `hb-script-list.h`.

use core::ffi::{c_char, c_int};

use crate::{HB_TAG, HB_TAG_NONE, hb_direction_t, hb_tag_t};

/// Data type for scripts.
///
/// Every script value is an [`hb_tag_t`] holding the four-letter code defined
/// by [ISO 15924](https://unicode.org/iso15924/). See also the Script (`sc`)
/// property of the Unicode Character Database.
///
/// The alias is a signed C `int` because the C enumeration ends with two
/// private sentinels equal to [`HB_TAG_MAX_SIGNED`](crate::HB_TAG_MAX_SIGNED),
/// which pins the underlying type. Those sentinels exist so that *any*
/// [`hb_tag_t`] bit pattern can be stored in an `hb_script_t` without undefined
/// behaviour: values are not restricted to the `HB_SCRIPT_*` constants below,
/// and text or font data can legitimately produce a tag that has no constant
/// here.
pub type hb_script_t = c_int;

/// Common (`Zyyy`) — the pseudo-script Unicode assigns to characters that
/// belong to no single script, such as spaces, digits, and most punctuation.
/// Unicode 1.1.
pub const HB_SCRIPT_COMMON: hb_script_t = HB_TAG(b'Z', b'y', b'y', b'y') as hb_script_t;
/// Inherited (`Zinh`) — the pseudo-script Unicode assigns to characters that
/// take their script from the preceding character, such as combining marks.
/// Unicode 1.1.
pub const HB_SCRIPT_INHERITED: hb_script_t = HB_TAG(b'Z', b'i', b'n', b'h') as hb_script_t;
/// Unknown (`Zzzz`) — the pseudo-script Unicode assigns to unassigned,
/// private-use, noncharacter, and surrogate code points. Unicode 5.0.
pub const HB_SCRIPT_UNKNOWN: hb_script_t = HB_TAG(b'Z', b'z', b'z', b'z') as hb_script_t;

/// Arabic (`Arab`) — Unicode 1.1.
pub const HB_SCRIPT_ARABIC: hb_script_t = HB_TAG(b'A', b'r', b'a', b'b') as hb_script_t;
/// Armenian (`Armn`) — Unicode 1.1.
pub const HB_SCRIPT_ARMENIAN: hb_script_t = HB_TAG(b'A', b'r', b'm', b'n') as hb_script_t;
/// Bengali (`Beng`) — Unicode 1.1.
pub const HB_SCRIPT_BENGALI: hb_script_t = HB_TAG(b'B', b'e', b'n', b'g') as hb_script_t;
/// Cyrillic (`Cyrl`) — Unicode 1.1.
pub const HB_SCRIPT_CYRILLIC: hb_script_t = HB_TAG(b'C', b'y', b'r', b'l') as hb_script_t;
/// Devanagari (`Deva`) — Unicode 1.1.
pub const HB_SCRIPT_DEVANAGARI: hb_script_t = HB_TAG(b'D', b'e', b'v', b'a') as hb_script_t;
/// Georgian (`Geor`) — Unicode 1.1.
pub const HB_SCRIPT_GEORGIAN: hb_script_t = HB_TAG(b'G', b'e', b'o', b'r') as hb_script_t;
/// Greek (`Grek`) — Unicode 1.1.
pub const HB_SCRIPT_GREEK: hb_script_t = HB_TAG(b'G', b'r', b'e', b'k') as hb_script_t;
/// Gujarati (`Gujr`) — Unicode 1.1.
pub const HB_SCRIPT_GUJARATI: hb_script_t = HB_TAG(b'G', b'u', b'j', b'r') as hb_script_t;
/// Gurmukhi (`Guru`) — Unicode 1.1.
pub const HB_SCRIPT_GURMUKHI: hb_script_t = HB_TAG(b'G', b'u', b'r', b'u') as hb_script_t;
/// Hangul (`Hang`) — Unicode 1.1.
pub const HB_SCRIPT_HANGUL: hb_script_t = HB_TAG(b'H', b'a', b'n', b'g') as hb_script_t;
/// Han (`Hani`) — Unicode 1.1.
pub const HB_SCRIPT_HAN: hb_script_t = HB_TAG(b'H', b'a', b'n', b'i') as hb_script_t;
/// Hebrew (`Hebr`) — Unicode 1.1.
pub const HB_SCRIPT_HEBREW: hb_script_t = HB_TAG(b'H', b'e', b'b', b'r') as hb_script_t;
/// Hiragana (`Hira`) — Unicode 1.1.
pub const HB_SCRIPT_HIRAGANA: hb_script_t = HB_TAG(b'H', b'i', b'r', b'a') as hb_script_t;
/// Kannada (`Knda`) — Unicode 1.1.
pub const HB_SCRIPT_KANNADA: hb_script_t = HB_TAG(b'K', b'n', b'd', b'a') as hb_script_t;
/// Katakana (`Kana`) — Unicode 1.1.
pub const HB_SCRIPT_KATAKANA: hb_script_t = HB_TAG(b'K', b'a', b'n', b'a') as hb_script_t;
/// Lao (`Laoo`) — Unicode 1.1.
pub const HB_SCRIPT_LAO: hb_script_t = HB_TAG(b'L', b'a', b'o', b'o') as hb_script_t;
/// Latin (`Latn`) — Unicode 1.1.
pub const HB_SCRIPT_LATIN: hb_script_t = HB_TAG(b'L', b'a', b't', b'n') as hb_script_t;
/// Malayalam (`Mlym`) — Unicode 1.1.
pub const HB_SCRIPT_MALAYALAM: hb_script_t = HB_TAG(b'M', b'l', b'y', b'm') as hb_script_t;
/// Oriya (`Orya`) — Unicode 1.1.
pub const HB_SCRIPT_ORIYA: hb_script_t = HB_TAG(b'O', b'r', b'y', b'a') as hb_script_t;
/// Tamil (`Taml`) — Unicode 1.1.
pub const HB_SCRIPT_TAMIL: hb_script_t = HB_TAG(b'T', b'a', b'm', b'l') as hb_script_t;
/// Telugu (`Telu`) — Unicode 1.1.
pub const HB_SCRIPT_TELUGU: hb_script_t = HB_TAG(b'T', b'e', b'l', b'u') as hb_script_t;
/// Thai (`Thai`) — Unicode 1.1.
pub const HB_SCRIPT_THAI: hb_script_t = HB_TAG(b'T', b'h', b'a', b'i') as hb_script_t;

/// Tibetan (`Tibt`) — Unicode 2.0.
pub const HB_SCRIPT_TIBETAN: hb_script_t = HB_TAG(b'T', b'i', b'b', b't') as hb_script_t;

/// Bopomofo (`Bopo`) — Unicode 3.0.
pub const HB_SCRIPT_BOPOMOFO: hb_script_t = HB_TAG(b'B', b'o', b'p', b'o') as hb_script_t;
/// Braille (`Brai`) — Unicode 3.0.
pub const HB_SCRIPT_BRAILLE: hb_script_t = HB_TAG(b'B', b'r', b'a', b'i') as hb_script_t;
/// Unified Canadian Aboriginal Syllabics (`Cans`) — Unicode 3.0.
pub const HB_SCRIPT_CANADIAN_SYLLABICS: hb_script_t = HB_TAG(b'C', b'a', b'n', b's') as hb_script_t;
/// Cherokee (`Cher`) — Unicode 3.0.
pub const HB_SCRIPT_CHEROKEE: hb_script_t = HB_TAG(b'C', b'h', b'e', b'r') as hb_script_t;
/// Ethiopic (`Ethi`) — Unicode 3.0.
pub const HB_SCRIPT_ETHIOPIC: hb_script_t = HB_TAG(b'E', b't', b'h', b'i') as hb_script_t;
/// Khmer (`Khmr`) — Unicode 3.0.
pub const HB_SCRIPT_KHMER: hb_script_t = HB_TAG(b'K', b'h', b'm', b'r') as hb_script_t;
/// Mongolian (`Mong`) — Unicode 3.0.
pub const HB_SCRIPT_MONGOLIAN: hb_script_t = HB_TAG(b'M', b'o', b'n', b'g') as hb_script_t;
/// Myanmar (`Mymr`) — Unicode 3.0.
pub const HB_SCRIPT_MYANMAR: hb_script_t = HB_TAG(b'M', b'y', b'm', b'r') as hb_script_t;
/// Ogham (`Ogam`) — Unicode 3.0.
pub const HB_SCRIPT_OGHAM: hb_script_t = HB_TAG(b'O', b'g', b'a', b'm') as hb_script_t;
/// Runic (`Runr`) — Unicode 3.0.
pub const HB_SCRIPT_RUNIC: hb_script_t = HB_TAG(b'R', b'u', b'n', b'r') as hb_script_t;
/// Sinhala (`Sinh`) — Unicode 3.0.
pub const HB_SCRIPT_SINHALA: hb_script_t = HB_TAG(b'S', b'i', b'n', b'h') as hb_script_t;
/// Syriac (`Syrc`) — Unicode 3.0.
pub const HB_SCRIPT_SYRIAC: hb_script_t = HB_TAG(b'S', b'y', b'r', b'c') as hb_script_t;
/// Thaana (`Thaa`) — Unicode 3.0.
pub const HB_SCRIPT_THAANA: hb_script_t = HB_TAG(b'T', b'h', b'a', b'a') as hb_script_t;
/// Yi (`Yiii`) — Unicode 3.0.
pub const HB_SCRIPT_YI: hb_script_t = HB_TAG(b'Y', b'i', b'i', b'i') as hb_script_t;

/// Deseret (`Dsrt`) — Unicode 3.1.
pub const HB_SCRIPT_DESERET: hb_script_t = HB_TAG(b'D', b's', b'r', b't') as hb_script_t;
/// Gothic (`Goth`) — Unicode 3.1.
pub const HB_SCRIPT_GOTHIC: hb_script_t = HB_TAG(b'G', b'o', b't', b'h') as hb_script_t;
/// Old Italic (`Ital`) — Unicode 3.1.
pub const HB_SCRIPT_OLD_ITALIC: hb_script_t = HB_TAG(b'I', b't', b'a', b'l') as hb_script_t;

/// Buhid (`Buhd`) — Unicode 3.2.
pub const HB_SCRIPT_BUHID: hb_script_t = HB_TAG(b'B', b'u', b'h', b'd') as hb_script_t;
/// Hanunoo (`Hano`) — Unicode 3.2.
pub const HB_SCRIPT_HANUNOO: hb_script_t = HB_TAG(b'H', b'a', b'n', b'o') as hb_script_t;
/// Tagalog (`Tglg`) — Unicode 3.2.
pub const HB_SCRIPT_TAGALOG: hb_script_t = HB_TAG(b'T', b'g', b'l', b'g') as hb_script_t;
/// Tagbanwa (`Tagb`) — Unicode 3.2.
pub const HB_SCRIPT_TAGBANWA: hb_script_t = HB_TAG(b'T', b'a', b'g', b'b') as hb_script_t;

/// Cypriot (`Cprt`) — Unicode 4.0.
pub const HB_SCRIPT_CYPRIOT: hb_script_t = HB_TAG(b'C', b'p', b'r', b't') as hb_script_t;
/// Limbu (`Limb`) — Unicode 4.0.
pub const HB_SCRIPT_LIMBU: hb_script_t = HB_TAG(b'L', b'i', b'm', b'b') as hb_script_t;
/// Linear B (`Linb`) — Unicode 4.0.
pub const HB_SCRIPT_LINEAR_B: hb_script_t = HB_TAG(b'L', b'i', b'n', b'b') as hb_script_t;
/// Osmanya (`Osma`) — Unicode 4.0.
pub const HB_SCRIPT_OSMANYA: hb_script_t = HB_TAG(b'O', b's', b'm', b'a') as hb_script_t;
/// Shavian (`Shaw`) — Unicode 4.0.
pub const HB_SCRIPT_SHAVIAN: hb_script_t = HB_TAG(b'S', b'h', b'a', b'w') as hb_script_t;
/// Tai Le (`Tale`) — Unicode 4.0.
pub const HB_SCRIPT_TAI_LE: hb_script_t = HB_TAG(b'T', b'a', b'l', b'e') as hb_script_t;
/// Ugaritic (`Ugar`) — Unicode 4.0.
pub const HB_SCRIPT_UGARITIC: hb_script_t = HB_TAG(b'U', b'g', b'a', b'r') as hb_script_t;

/// Buginese (`Bugi`) — Unicode 4.1.
pub const HB_SCRIPT_BUGINESE: hb_script_t = HB_TAG(b'B', b'u', b'g', b'i') as hb_script_t;
/// Coptic (`Copt`) — Unicode 4.1.
pub const HB_SCRIPT_COPTIC: hb_script_t = HB_TAG(b'C', b'o', b'p', b't') as hb_script_t;
/// Glagolitic (`Glag`) — Unicode 4.1.
pub const HB_SCRIPT_GLAGOLITIC: hb_script_t = HB_TAG(b'G', b'l', b'a', b'g') as hb_script_t;
/// Kharoshthi (`Khar`) — Unicode 4.1.
pub const HB_SCRIPT_KHAROSHTHI: hb_script_t = HB_TAG(b'K', b'h', b'a', b'r') as hb_script_t;
/// New Tai Lue (`Talu`) — Unicode 4.1.
pub const HB_SCRIPT_NEW_TAI_LUE: hb_script_t = HB_TAG(b'T', b'a', b'l', b'u') as hb_script_t;
/// Old Persian (`Xpeo`) — Unicode 4.1.
pub const HB_SCRIPT_OLD_PERSIAN: hb_script_t = HB_TAG(b'X', b'p', b'e', b'o') as hb_script_t;
/// Syloti Nagri (`Sylo`) — Unicode 4.1.
pub const HB_SCRIPT_SYLOTI_NAGRI: hb_script_t = HB_TAG(b'S', b'y', b'l', b'o') as hb_script_t;
/// Tifinagh (`Tfng`) — Unicode 4.1.
pub const HB_SCRIPT_TIFINAGH: hb_script_t = HB_TAG(b'T', b'f', b'n', b'g') as hb_script_t;

/// Balinese (`Bali`) — Unicode 5.0.
pub const HB_SCRIPT_BALINESE: hb_script_t = HB_TAG(b'B', b'a', b'l', b'i') as hb_script_t;
/// Cuneiform (`Xsux`) — Unicode 5.0.
pub const HB_SCRIPT_CUNEIFORM: hb_script_t = HB_TAG(b'X', b's', b'u', b'x') as hb_script_t;
/// N'Ko (`Nkoo`) — Unicode 5.0.
pub const HB_SCRIPT_NKO: hb_script_t = HB_TAG(b'N', b'k', b'o', b'o') as hb_script_t;
/// Phags-pa (`Phag`) — Unicode 5.0.
pub const HB_SCRIPT_PHAGS_PA: hb_script_t = HB_TAG(b'P', b'h', b'a', b'g') as hb_script_t;
/// Phoenician (`Phnx`) — Unicode 5.0.
pub const HB_SCRIPT_PHOENICIAN: hb_script_t = HB_TAG(b'P', b'h', b'n', b'x') as hb_script_t;

/// Carian (`Cari`) — Unicode 5.1.
pub const HB_SCRIPT_CARIAN: hb_script_t = HB_TAG(b'C', b'a', b'r', b'i') as hb_script_t;
/// Cham (`Cham`) — Unicode 5.1.
pub const HB_SCRIPT_CHAM: hb_script_t = HB_TAG(b'C', b'h', b'a', b'm') as hb_script_t;
/// Kayah Li (`Kali`) — Unicode 5.1.
pub const HB_SCRIPT_KAYAH_LI: hb_script_t = HB_TAG(b'K', b'a', b'l', b'i') as hb_script_t;
/// Lepcha (`Lepc`) — Unicode 5.1.
pub const HB_SCRIPT_LEPCHA: hb_script_t = HB_TAG(b'L', b'e', b'p', b'c') as hb_script_t;
/// Lycian (`Lyci`) — Unicode 5.1.
pub const HB_SCRIPT_LYCIAN: hb_script_t = HB_TAG(b'L', b'y', b'c', b'i') as hb_script_t;
/// Lydian (`Lydi`) — Unicode 5.1.
pub const HB_SCRIPT_LYDIAN: hb_script_t = HB_TAG(b'L', b'y', b'd', b'i') as hb_script_t;
/// Ol Chiki (`Olck`) — Unicode 5.1.
pub const HB_SCRIPT_OL_CHIKI: hb_script_t = HB_TAG(b'O', b'l', b'c', b'k') as hb_script_t;
/// Rejang (`Rjng`) — Unicode 5.1.
pub const HB_SCRIPT_REJANG: hb_script_t = HB_TAG(b'R', b'j', b'n', b'g') as hb_script_t;
/// Saurashtra (`Saur`) — Unicode 5.1.
pub const HB_SCRIPT_SAURASHTRA: hb_script_t = HB_TAG(b'S', b'a', b'u', b'r') as hb_script_t;
/// Sundanese (`Sund`) — Unicode 5.1.
pub const HB_SCRIPT_SUNDANESE: hb_script_t = HB_TAG(b'S', b'u', b'n', b'd') as hb_script_t;
/// Vai (`Vaii`) — Unicode 5.1.
pub const HB_SCRIPT_VAI: hb_script_t = HB_TAG(b'V', b'a', b'i', b'i') as hb_script_t;

/// Avestan (`Avst`) — Unicode 5.2.
pub const HB_SCRIPT_AVESTAN: hb_script_t = HB_TAG(b'A', b'v', b's', b't') as hb_script_t;
/// Bamum (`Bamu`) — Unicode 5.2.
pub const HB_SCRIPT_BAMUM: hb_script_t = HB_TAG(b'B', b'a', b'm', b'u') as hb_script_t;
/// Egyptian Hieroglyphs (`Egyp`) — Unicode 5.2.
pub const HB_SCRIPT_EGYPTIAN_HIEROGLYPHS: hb_script_t =
    HB_TAG(b'E', b'g', b'y', b'p') as hb_script_t;
/// Imperial Aramaic (`Armi`) — Unicode 5.2.
pub const HB_SCRIPT_IMPERIAL_ARAMAIC: hb_script_t = HB_TAG(b'A', b'r', b'm', b'i') as hb_script_t;
/// Inscriptional Pahlavi (`Phli`) — Unicode 5.2.
pub const HB_SCRIPT_INSCRIPTIONAL_PAHLAVI: hb_script_t =
    HB_TAG(b'P', b'h', b'l', b'i') as hb_script_t;
/// Inscriptional Parthian (`Prti`) — Unicode 5.2.
pub const HB_SCRIPT_INSCRIPTIONAL_PARTHIAN: hb_script_t =
    HB_TAG(b'P', b'r', b't', b'i') as hb_script_t;
/// Javanese (`Java`) — Unicode 5.2.
pub const HB_SCRIPT_JAVANESE: hb_script_t = HB_TAG(b'J', b'a', b'v', b'a') as hb_script_t;
/// Kaithi (`Kthi`) — Unicode 5.2.
pub const HB_SCRIPT_KAITHI: hb_script_t = HB_TAG(b'K', b't', b'h', b'i') as hb_script_t;
/// Lisu (`Lisu`) — Unicode 5.2.
pub const HB_SCRIPT_LISU: hb_script_t = HB_TAG(b'L', b'i', b's', b'u') as hb_script_t;
/// Meetei Mayek (`Mtei`) — Unicode 5.2.
pub const HB_SCRIPT_MEETEI_MAYEK: hb_script_t = HB_TAG(b'M', b't', b'e', b'i') as hb_script_t;
/// Old South Arabian (`Sarb`) — Unicode 5.2.
pub const HB_SCRIPT_OLD_SOUTH_ARABIAN: hb_script_t = HB_TAG(b'S', b'a', b'r', b'b') as hb_script_t;
/// Old Turkic (`Orkh`) — Unicode 5.2.
pub const HB_SCRIPT_OLD_TURKIC: hb_script_t = HB_TAG(b'O', b'r', b'k', b'h') as hb_script_t;
/// Samaritan (`Samr`) — Unicode 5.2.
pub const HB_SCRIPT_SAMARITAN: hb_script_t = HB_TAG(b'S', b'a', b'm', b'r') as hb_script_t;
/// Tai Tham (`Lana`) — Unicode 5.2.
pub const HB_SCRIPT_TAI_THAM: hb_script_t = HB_TAG(b'L', b'a', b'n', b'a') as hb_script_t;
/// Tai Viet (`Tavt`) — Unicode 5.2.
pub const HB_SCRIPT_TAI_VIET: hb_script_t = HB_TAG(b'T', b'a', b'v', b't') as hb_script_t;

/// Batak (`Batk`) — Unicode 6.0.
pub const HB_SCRIPT_BATAK: hb_script_t = HB_TAG(b'B', b'a', b't', b'k') as hb_script_t;
/// Brahmi (`Brah`) — Unicode 6.0.
pub const HB_SCRIPT_BRAHMI: hb_script_t = HB_TAG(b'B', b'r', b'a', b'h') as hb_script_t;
/// Mandaic (`Mand`) — Unicode 6.0.
pub const HB_SCRIPT_MANDAIC: hb_script_t = HB_TAG(b'M', b'a', b'n', b'd') as hb_script_t;

/// Chakma (`Cakm`) — Unicode 6.1.
pub const HB_SCRIPT_CHAKMA: hb_script_t = HB_TAG(b'C', b'a', b'k', b'm') as hb_script_t;
/// Meroitic Cursive (`Merc`) — Unicode 6.1.
pub const HB_SCRIPT_MEROITIC_CURSIVE: hb_script_t = HB_TAG(b'M', b'e', b'r', b'c') as hb_script_t;
/// Meroitic Hieroglyphs (`Mero`) — Unicode 6.1.
pub const HB_SCRIPT_MEROITIC_HIEROGLYPHS: hb_script_t =
    HB_TAG(b'M', b'e', b'r', b'o') as hb_script_t;
/// Miao (`Plrd`) — Unicode 6.1.
pub const HB_SCRIPT_MIAO: hb_script_t = HB_TAG(b'P', b'l', b'r', b'd') as hb_script_t;
/// Sharada (`Shrd`) — Unicode 6.1.
pub const HB_SCRIPT_SHARADA: hb_script_t = HB_TAG(b'S', b'h', b'r', b'd') as hb_script_t;
/// Sora Sompeng (`Sora`) — Unicode 6.1.
pub const HB_SCRIPT_SORA_SOMPENG: hb_script_t = HB_TAG(b'S', b'o', b'r', b'a') as hb_script_t;
/// Takri (`Takr`) — Unicode 6.1.
pub const HB_SCRIPT_TAKRI: hb_script_t = HB_TAG(b'T', b'a', b'k', b'r') as hb_script_t;

/// Bassa Vah (`Bass`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_BASSA_VAH: hb_script_t = HB_TAG(b'B', b'a', b's', b's') as hb_script_t;
/// Caucasian Albanian (`Aghb`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_CAUCASIAN_ALBANIAN: hb_script_t = HB_TAG(b'A', b'g', b'h', b'b') as hb_script_t;
/// Duployan (`Dupl`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_DUPLOYAN: hb_script_t = HB_TAG(b'D', b'u', b'p', b'l') as hb_script_t;
/// Elbasan (`Elba`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_ELBASAN: hb_script_t = HB_TAG(b'E', b'l', b'b', b'a') as hb_script_t;
/// Grantha (`Gran`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_GRANTHA: hb_script_t = HB_TAG(b'G', b'r', b'a', b'n') as hb_script_t;
/// Khojki (`Khoj`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_KHOJKI: hb_script_t = HB_TAG(b'K', b'h', b'o', b'j') as hb_script_t;
/// Khudawadi (`Sind`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_KHUDAWADI: hb_script_t = HB_TAG(b'S', b'i', b'n', b'd') as hb_script_t;
/// Linear A (`Lina`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_LINEAR_A: hb_script_t = HB_TAG(b'L', b'i', b'n', b'a') as hb_script_t;
/// Mahajani (`Mahj`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_MAHAJANI: hb_script_t = HB_TAG(b'M', b'a', b'h', b'j') as hb_script_t;
/// Manichaean (`Mani`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_MANICHAEAN: hb_script_t = HB_TAG(b'M', b'a', b'n', b'i') as hb_script_t;
/// Mende Kikakui (`Mend`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_MENDE_KIKAKUI: hb_script_t = HB_TAG(b'M', b'e', b'n', b'd') as hb_script_t;
/// Modi (`Modi`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_MODI: hb_script_t = HB_TAG(b'M', b'o', b'd', b'i') as hb_script_t;
/// Mro (`Mroo`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_MRO: hb_script_t = HB_TAG(b'M', b'r', b'o', b'o') as hb_script_t;
/// Nabataean (`Nbat`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_NABATAEAN: hb_script_t = HB_TAG(b'N', b'b', b'a', b't') as hb_script_t;
/// Old North Arabian (`Narb`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_OLD_NORTH_ARABIAN: hb_script_t = HB_TAG(b'N', b'a', b'r', b'b') as hb_script_t;
/// Old Permic (`Perm`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_OLD_PERMIC: hb_script_t = HB_TAG(b'P', b'e', b'r', b'm') as hb_script_t;
/// Pahawh Hmong (`Hmng`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_PAHAWH_HMONG: hb_script_t = HB_TAG(b'H', b'm', b'n', b'g') as hb_script_t;
/// Palmyrene (`Palm`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_PALMYRENE: hb_script_t = HB_TAG(b'P', b'a', b'l', b'm') as hb_script_t;
/// Pau Cin Hau (`Pauc`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_PAU_CIN_HAU: hb_script_t = HB_TAG(b'P', b'a', b'u', b'c') as hb_script_t;
/// Psalter Pahlavi (`Phlp`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_PSALTER_PAHLAVI: hb_script_t = HB_TAG(b'P', b'h', b'l', b'p') as hb_script_t;
/// Siddham (`Sidd`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_SIDDHAM: hb_script_t = HB_TAG(b'S', b'i', b'd', b'd') as hb_script_t;
/// Tirhuta (`Tirh`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_TIRHUTA: hb_script_t = HB_TAG(b'T', b'i', b'r', b'h') as hb_script_t;
/// Warang Citi (`Wara`) — Unicode 7.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_WARANG_CITI: hb_script_t = HB_TAG(b'W', b'a', b'r', b'a') as hb_script_t;

/// Ahom (`Ahom`) — Unicode 8.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_AHOM: hb_script_t = HB_TAG(b'A', b'h', b'o', b'm') as hb_script_t;
/// Anatolian Hieroglyphs (`Hluw`) — Unicode 8.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_ANATOLIAN_HIEROGLYPHS: hb_script_t =
    HB_TAG(b'H', b'l', b'u', b'w') as hb_script_t;
/// Hatran (`Hatr`) — Unicode 8.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_HATRAN: hb_script_t = HB_TAG(b'H', b'a', b't', b'r') as hb_script_t;
/// Multani (`Mult`) — Unicode 8.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_MULTANI: hb_script_t = HB_TAG(b'M', b'u', b'l', b't') as hb_script_t;
/// Old Hungarian (`Hung`) — Unicode 8.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_OLD_HUNGARIAN: hb_script_t = HB_TAG(b'H', b'u', b'n', b'g') as hb_script_t;
/// SignWriting (`Sgnw`) — Unicode 8.0.
///
/// Since HarfBuzz 0.9.30.
pub const HB_SCRIPT_SIGNWRITING: hb_script_t = HB_TAG(b'S', b'g', b'n', b'w') as hb_script_t;

/// Adlam (`Adlm`) — Unicode 9.0.
///
/// Since HarfBuzz 1.3.0.
pub const HB_SCRIPT_ADLAM: hb_script_t = HB_TAG(b'A', b'd', b'l', b'm') as hb_script_t;
/// Bhaiksuki (`Bhks`) — Unicode 9.0.
///
/// Since HarfBuzz 1.3.0.
pub const HB_SCRIPT_BHAIKSUKI: hb_script_t = HB_TAG(b'B', b'h', b'k', b's') as hb_script_t;
/// Marchen (`Marc`) — Unicode 9.0.
///
/// Since HarfBuzz 1.3.0.
pub const HB_SCRIPT_MARCHEN: hb_script_t = HB_TAG(b'M', b'a', b'r', b'c') as hb_script_t;
/// Osage (`Osge`) — Unicode 9.0.
///
/// Since HarfBuzz 1.3.0.
pub const HB_SCRIPT_OSAGE: hb_script_t = HB_TAG(b'O', b's', b'g', b'e') as hb_script_t;
/// Tangut (`Tang`) — Unicode 9.0.
///
/// Since HarfBuzz 1.3.0.
pub const HB_SCRIPT_TANGUT: hb_script_t = HB_TAG(b'T', b'a', b'n', b'g') as hb_script_t;
/// Newa (`Newa`) — Unicode 9.0.
///
/// Since HarfBuzz 1.3.0.
pub const HB_SCRIPT_NEWA: hb_script_t = HB_TAG(b'N', b'e', b'w', b'a') as hb_script_t;

/// Masaram Gondi (`Gonm`) — Unicode 10.0.
///
/// Since HarfBuzz 1.6.0.
pub const HB_SCRIPT_MASARAM_GONDI: hb_script_t = HB_TAG(b'G', b'o', b'n', b'm') as hb_script_t;
/// Nushu (`Nshu`) — Unicode 10.0.
///
/// Since HarfBuzz 1.6.0.
pub const HB_SCRIPT_NUSHU: hb_script_t = HB_TAG(b'N', b's', b'h', b'u') as hb_script_t;
/// Soyombo (`Soyo`) — Unicode 10.0.
///
/// Since HarfBuzz 1.6.0.
pub const HB_SCRIPT_SOYOMBO: hb_script_t = HB_TAG(b'S', b'o', b'y', b'o') as hb_script_t;
/// Zanabazar Square (`Zanb`) — Unicode 10.0.
///
/// Since HarfBuzz 1.6.0.
pub const HB_SCRIPT_ZANABAZAR_SQUARE: hb_script_t = HB_TAG(b'Z', b'a', b'n', b'b') as hb_script_t;

/// Dogra (`Dogr`) — Unicode 11.0.
///
/// Since HarfBuzz 1.8.0.
pub const HB_SCRIPT_DOGRA: hb_script_t = HB_TAG(b'D', b'o', b'g', b'r') as hb_script_t;
/// Gunjala Gondi (`Gong`) — Unicode 11.0.
///
/// Since HarfBuzz 1.8.0.
pub const HB_SCRIPT_GUNJALA_GONDI: hb_script_t = HB_TAG(b'G', b'o', b'n', b'g') as hb_script_t;
/// Hanifi Rohingya (`Rohg`) — Unicode 11.0.
///
/// Since HarfBuzz 1.8.0.
pub const HB_SCRIPT_HANIFI_ROHINGYA: hb_script_t = HB_TAG(b'R', b'o', b'h', b'g') as hb_script_t;
/// Makasar (`Maka`) — Unicode 11.0.
///
/// Since HarfBuzz 1.8.0.
pub const HB_SCRIPT_MAKASAR: hb_script_t = HB_TAG(b'M', b'a', b'k', b'a') as hb_script_t;
/// Medefaidrin (`Medf`) — Unicode 11.0.
///
/// Since HarfBuzz 1.8.0.
pub const HB_SCRIPT_MEDEFAIDRIN: hb_script_t = HB_TAG(b'M', b'e', b'd', b'f') as hb_script_t;
/// Old Sogdian (`Sogo`) — Unicode 11.0.
///
/// Since HarfBuzz 1.8.0.
pub const HB_SCRIPT_OLD_SOGDIAN: hb_script_t = HB_TAG(b'S', b'o', b'g', b'o') as hb_script_t;
/// Sogdian (`Sogd`) — Unicode 11.0.
///
/// Since HarfBuzz 1.8.0.
pub const HB_SCRIPT_SOGDIAN: hb_script_t = HB_TAG(b'S', b'o', b'g', b'd') as hb_script_t;

/// Elymaic (`Elym`) — Unicode 12.0.
///
/// Since HarfBuzz 2.4.0.
pub const HB_SCRIPT_ELYMAIC: hb_script_t = HB_TAG(b'E', b'l', b'y', b'm') as hb_script_t;
/// Nandinagari (`Nand`) — Unicode 12.0.
///
/// Since HarfBuzz 2.4.0.
pub const HB_SCRIPT_NANDINAGARI: hb_script_t = HB_TAG(b'N', b'a', b'n', b'd') as hb_script_t;
/// Nyiakeng Puachue Hmong (`Hmnp`) — Unicode 12.0.
///
/// Since HarfBuzz 2.4.0.
pub const HB_SCRIPT_NYIAKENG_PUACHUE_HMONG: hb_script_t =
    HB_TAG(b'H', b'm', b'n', b'p') as hb_script_t;
/// Wancho (`Wcho`) — Unicode 12.0.
///
/// Since HarfBuzz 2.4.0.
pub const HB_SCRIPT_WANCHO: hb_script_t = HB_TAG(b'W', b'c', b'h', b'o') as hb_script_t;

/// Chorasmian (`Chrs`) — Unicode 13.0.
///
/// Since HarfBuzz 2.6.7.
pub const HB_SCRIPT_CHORASMIAN: hb_script_t = HB_TAG(b'C', b'h', b'r', b's') as hb_script_t;
/// Dives Akuru (`Diak`) — Unicode 13.0.
///
/// Since HarfBuzz 2.6.7.
pub const HB_SCRIPT_DIVES_AKURU: hb_script_t = HB_TAG(b'D', b'i', b'a', b'k') as hb_script_t;
/// Khitan Small Script (`Kits`) — Unicode 13.0.
///
/// Since HarfBuzz 2.6.7.
pub const HB_SCRIPT_KHITAN_SMALL_SCRIPT: hb_script_t =
    HB_TAG(b'K', b'i', b't', b's') as hb_script_t;
/// Yezidi (`Yezi`) — Unicode 13.0.
///
/// Since HarfBuzz 2.6.7.
pub const HB_SCRIPT_YEZIDI: hb_script_t = HB_TAG(b'Y', b'e', b'z', b'i') as hb_script_t;

/// Cypro-Minoan (`Cpmn`) — Unicode 14.0.
///
/// Since HarfBuzz 3.0.0.
pub const HB_SCRIPT_CYPRO_MINOAN: hb_script_t = HB_TAG(b'C', b'p', b'm', b'n') as hb_script_t;
/// Old Uyghur (`Ougr`) — Unicode 14.0.
///
/// Since HarfBuzz 3.0.0.
pub const HB_SCRIPT_OLD_UYGHUR: hb_script_t = HB_TAG(b'O', b'u', b'g', b'r') as hb_script_t;
/// Tangsa (`Tnsa`) — Unicode 14.0.
///
/// Since HarfBuzz 3.0.0.
pub const HB_SCRIPT_TANGSA: hb_script_t = HB_TAG(b'T', b'n', b's', b'a') as hb_script_t;
/// Toto (`Toto`) — Unicode 14.0.
///
/// Since HarfBuzz 3.0.0.
pub const HB_SCRIPT_TOTO: hb_script_t = HB_TAG(b'T', b'o', b't', b'o') as hb_script_t;
/// Vithkuqi (`Vith`) — Unicode 14.0.
///
/// Since HarfBuzz 3.0.0.
pub const HB_SCRIPT_VITHKUQI: hb_script_t = HB_TAG(b'V', b'i', b't', b'h') as hb_script_t;

/// Mathematical notation (`Zmth`) — a pseudo-script, not an ISO 15924 script
/// proper.
///
/// Since HarfBuzz 3.4.0.
pub const HB_SCRIPT_MATH: hb_script_t = HB_TAG(b'Z', b'm', b't', b'h') as hb_script_t;

/// Kawi (`Kawi`) — Unicode 15.0.
///
/// Since HarfBuzz 5.2.0.
pub const HB_SCRIPT_KAWI: hb_script_t = HB_TAG(b'K', b'a', b'w', b'i') as hb_script_t;
/// Nag Mundari (`Nagm`) — Unicode 15.0.
///
/// Since HarfBuzz 5.2.0.
pub const HB_SCRIPT_NAG_MUNDARI: hb_script_t = HB_TAG(b'N', b'a', b'g', b'm') as hb_script_t;

/// Garay (`Gara`) — Unicode 16.0.
///
/// Since HarfBuzz 10.0.0.
pub const HB_SCRIPT_GARAY: hb_script_t = HB_TAG(b'G', b'a', b'r', b'a') as hb_script_t;
/// Gurung Khema (`Gukh`) — Unicode 16.0.
///
/// Since HarfBuzz 10.0.0.
pub const HB_SCRIPT_GURUNG_KHEMA: hb_script_t = HB_TAG(b'G', b'u', b'k', b'h') as hb_script_t;
/// Kirat Rai (`Krai`) — Unicode 16.0.
///
/// Since HarfBuzz 10.0.0.
pub const HB_SCRIPT_KIRAT_RAI: hb_script_t = HB_TAG(b'K', b'r', b'a', b'i') as hb_script_t;
/// Ol Onal (`Onao`) — Unicode 16.0.
///
/// Since HarfBuzz 10.0.0.
pub const HB_SCRIPT_OL_ONAL: hb_script_t = HB_TAG(b'O', b'n', b'a', b'o') as hb_script_t;
/// Sunuwar (`Sunu`) — Unicode 16.0.
///
/// Since HarfBuzz 10.0.0.
pub const HB_SCRIPT_SUNUWAR: hb_script_t = HB_TAG(b'S', b'u', b'n', b'u') as hb_script_t;
/// Todhri (`Todr`) — Unicode 16.0.
///
/// Since HarfBuzz 10.0.0.
pub const HB_SCRIPT_TODHRI: hb_script_t = HB_TAG(b'T', b'o', b'd', b'r') as hb_script_t;
/// Tulu-Tigalari (`Tutg`) — Unicode 16.0.
///
/// Since HarfBuzz 10.0.0.
pub const HB_SCRIPT_TULU_TIGALARI: hb_script_t = HB_TAG(b'T', b'u', b't', b'g') as hb_script_t;

/// Beria Erfe (`Berf`) — Unicode 17.0.
///
/// Since HarfBuzz 11.5.0.
pub const HB_SCRIPT_BERIA_ERFE: hb_script_t = HB_TAG(b'B', b'e', b'r', b'f') as hb_script_t;
/// Sidetic (`Sidt`) — Unicode 17.0.
///
/// Since HarfBuzz 11.5.0.
pub const HB_SCRIPT_SIDETIC: hb_script_t = HB_TAG(b'S', b'i', b'd', b't') as hb_script_t;
/// Tai Yo (`Tayo`) — Unicode 17.0.
///
/// Since HarfBuzz 11.5.0.
pub const HB_SCRIPT_TAI_YO: hb_script_t = HB_TAG(b'T', b'a', b'y', b'o') as hb_script_t;
/// Tolong Siki (`Tols`) — Unicode 17.0.
///
/// Since HarfBuzz 11.5.0.
pub const HB_SCRIPT_TOLONG_SIKI: hb_script_t = HB_TAG(b'T', b'o', b'l', b's') as hb_script_t;

/// No script set.
///
/// Numerically equal to [`HB_TAG_NONE`], i.e. zero.
pub const HB_SCRIPT_INVALID: hb_script_t = HB_TAG_NONE as hb_script_t;

unsafe extern "C" {
    /// Converts an ISO 15924 script tag to the corresponding script.
    ///
    /// The match is case-insensitive, a handful of historic and variant tags
    /// are folded onto their modern script (`Qaai` to [`HB_SCRIPT_INHERITED`],
    /// `Hans` and `Hant` to [`HB_SCRIPT_HAN`], and so on), and a tag that
    /// merely *looks* like a script code is passed through unchanged. Anything
    /// else becomes [`HB_SCRIPT_UNKNOWN`]; [`HB_TAG_NONE`] becomes
    /// [`HB_SCRIPT_INVALID`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_script_from_iso15924_tag(tag: hb_tag_t) -> hb_script_t;

    /// Converts a string holding an ISO 15924 script tag to the corresponding
    /// script.
    ///
    /// Shorthand for [`hb_tag_from_string`](crate::hb_tag_from_string) followed
    /// by [`hb_script_from_iso15924_tag`]. Pass `len` as `-1` when `str_` is
    /// NUL-terminated.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_script_from_string(str_: *const c_char, len: c_int) -> hb_script_t;

    /// Converts a script to the corresponding ISO 15924 script tag.
    ///
    /// This is a plain reinterpretation of the value, so it never fails and
    /// round-trips any tag that was passed through
    /// [`hb_script_from_iso15924_tag`] unchanged.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_script_to_iso15924_tag(script: hb_script_t) -> hb_tag_t;

    /// Fetches the direction a script is written in when set horizontally.
    ///
    /// Right-to-left scripts return
    /// [`HB_DIRECTION_RTL`](crate::HB_DIRECTION_RTL) and left-to-right scripts
    /// return [`HB_DIRECTION_LTR`](crate::HB_DIRECTION_LTR). Scripts that may
    /// be written in either direction return
    /// [`HB_DIRECTION_INVALID`](crate::HB_DIRECTION_INVALID), and unrecognized
    /// scripts return [`HB_DIRECTION_LTR`](crate::HB_DIRECTION_LTR).
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_script_get_horizontal_direction(script: hb_script_t) -> hb_direction_t;
}
