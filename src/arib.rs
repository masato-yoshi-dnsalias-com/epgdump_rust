
use encoding_rs::ISO_2022_JP;
use std::sync::atomic::{AtomicI32, AtomicU32, Ordering::*};

// グラフィックセット
const CODE_UNKNOWN: u32            = 0;  // 不明なグラフィックセット(非対応)
const CODE_KANJI: u32              = 1;  // Kanji
const CODE_ALPHANUMERIC: u32       = 2;  // Alphanumeric
const CODE_HIRAGANA: u32           = 3;  // Hiragana
const CODE_KATAKANA: u32           = 4;  // Katakana
const CODE_MOSAIC_A: u32           = 5;  // Mosaic A
const CODE_MOSAIC_B: u32           = 6;  // Mosaic B
const CODE_MOSAIC_C: u32           = 7;  // Mosaic C
const CODE_MOSAIC_D: u32           = 8;  // Mosaic D
const CODE_PROP_ALPHANUMERIC: u32  = 9;  // Proportional Alphanumeric
const CODE_PROP_HIRAGANA: u32      = 10; // Proportional Hiragana
const CODE_PROP_KATAKANA: u32      = 11; // Proportional Katakana
const CODE_JIS_X0201_KATAKANA: u32 = 12; // JIS X 0201 Katakana
const CODE_JIS_KANJI_PLANE_1: u32  = 13; // JIS compatible Kanji Plane 1
const CODE_JIS_KANJI_PLANE_2: u32  = 14; // JIS compatible Kanji Plane 2
const CODE_ADDITIONAL_SYMBOLS: u32 = 15; // Additional symbols

// グラフィックセットのキャラクターサイズテーブル
const AB_CHAR_SIZE_TABLE: [bool; 16] = [
    false, // CODE_UNKNOWN               不明なグラフィックセット(非対応)
    true,  // CODE_KANJI                 Kanji
    false, // CODE_ALPHANUMERIC          Alphanumeric
    false, // CODE_HIRAGANA              Hiragana
    false, // CODE_KATAKANA              Katakana
    false, // CODE_MOSAIC_A              Mosaic A
    false, // CODE_MOSAIC_B              Mosaic B
    false, // CODE_MOSAIC_C              Mosaic C
    false, // CODE_MOSAIC_D              Mosaic D
    false, // CODE_PROP_ALPHANUMERIC     Proportional Alphanumeric
    false, // CODE_PROP_HIRAGANA         Proportional Hiragana
    false, // CODE_PROP_KATAKANA         Proportional Katakana
    false, // CODE_JIS_X0201_KATAKANA    JIS X 0201 Katakana
    true,  // CODE_JIS_KANJI_PLANE_1     JIS compatible Kanji Plane 1
    true,  // CODE_JIS_KANJI_PLANE_2     JIS compatible Kanji Plane 2
    true,  // CODE_ADDITIONAL_SYMBOLS    Additional symbols
];

// キャラクターサイズ
#[allow(dead_code)]
const STR_SMALL: i32     = 0;
#[allow(dead_code)]
const STR_MEDIUM: i32    = 1;
#[allow(dead_code)]
const STR_NORMAL: i32    = 2;
#[allow(dead_code)]
const STR_MICRO: i32     = 3;
#[allow(dead_code)]
const STR_HIGH_W: i32    = 4;
#[allow(dead_code)]
const STR_WIDTH_W: i32   = 5;
#[allow(dead_code)]
const STR_W: i32         = 6;
#[allow(dead_code)]
const STR_SPECIAL_1: i32 = 7;
#[allow(dead_code)]
const STR_SPECIAL_2: i32 = 8;

// ESCカウンター
static ESC_SEQ_COUNT: AtomicU32 =  AtomicU32::new(0);

// グラフィックセットカウンター
/*static M_CODE_G: [AtomicU32; 4] = [
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
];*/

#[allow(dead_code,unused_variables)]
static M_LOCKING_GL: AtomicU32 = AtomicU32::new(0);
#[allow(dead_code,unused_variables)]
static M_LOCKING_GR: AtomicU32 = AtomicU32::new(0);
#[allow(dead_code,unused_variables)]
static M_SINGLE_GL: AtomicU32 = AtomicU32::new(0);
#[allow(dead_code,unused_variables)]
static M_EM_STR_SIZE: AtomicU32 = AtomicU32::new(0);

// GLカウンター
static GL_NUM: AtomicU32 = AtomicU32::new(0);

// GRカウンター
static GR_NUM: AtomicU32 = AtomicU32::new(0);

// グラフィックページ
static G_PAGE: [AtomicU32; 4] = [
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
];

// 文字サイズ
static STR_SIZE: AtomicI32 = AtomicI32::new(0);

// マルチバイト文字の処理(リターン:文字(STring))
fn multi_byte(g_code: u32, c1: u8, c2: u8) -> String {

    // JISコードワーク
    let mut work_code: [u8; 8] = [0; 8];

    // g_code毎の処理
    match g_code {

        // 漢字、ＪＩＳ漢字１、ＪＩＳ漢字２の処理
        CODE_KANJI | CODE_JIS_KANJI_PLANE_1 | CODE_JIS_KANJI_PLANE_2 => {

            // JISコード作成
            work_code[0] = 0x1b; //
            work_code[1] = 0x24; // 漢字ＩＮ
            work_code[2] = 0x42; //
            work_code[3] = c1;   // 漢字コード（１バイト目）
            work_code[4] = c2;   // 漢字コード（２バイト目）
            work_code[5] = 0x1b; //
            work_code[6] = 0x28; // 漢字ＯＵＴ
            work_code[7] = 0x42; //

            // JIS to UTF-8変換
            let (ret_code, _, _) = ISO_2022_JP.decode(&work_code);

            // リターン情報
            ret_code.to_string()

        },
        CODE_ADDITIONAL_SYMBOLS =>{

            // シンボル文字変換
            let ret_str = put_symbols_char(((c1 as u32) << 8) + c2 as u32);

            // リターン情報
            ret_str.to_string()

        }
        _ => {

            // リターン情報(デフォルトは文字なし)
            "".to_string()

        },
    }

}

const AC_ALPHANUMERIC_TABLE:[&str; 0x80] = [
    "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　",
    "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　",
    "　", "！", "”", "＃", "＄", "％", "＆", "’", "（", "）", "＊", "＋", "，", "－", "．", "／",
    "０", "１", "２", "３", "４", "５", "６", "７", "８", "９", "：", "；", "＜", "＝", "＞", "？",
    "＠", "Ａ", "Ｂ", "Ｃ", "Ｄ", "Ｅ", "Ｆ", "Ｇ", "Ｈ", "Ｉ", "Ｊ", "Ｋ", "Ｌ", "Ｍ", "Ｎ", "Ｏ",
    "Ｐ", "Ｑ", "Ｒ", "Ｓ", "Ｔ", "Ｕ", "Ｖ", "Ｗ", "Ｘ", "Ｙ", "Ｚ", "［", "￥", "］", "＾", "＿",
    "　", "ａ", "ｂ", "ｃ", "ｄ", "ｅ", "ｆ", "ｇ", "ｈ", "ｉ", "ｊ", "ｋ", "ｌ", "ｍ", "ｎ", "ｏ",
    "ｐ", "ｑ", "ｒ", "ｓ", "ｔ", "ｕ", "ｖ", "ｗ", "ｘ", "ｙ", "ｚ", "｛", "｜", "｝", "￣", "　",
];

// 英数字コード -> 文字変換処理(リターン:文字(&str))
pub fn put_alphanumeric_char(code: u8) -> &'static str {

    // リターン情報(AC_ALPHANUMERIC_TABLEの内容)
    AC_ALPHANUMERIC_TABLE[code as usize]

}

const AC_HIRAGANA_TABLE:[&str; 0x80] = [
    "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　",
    "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　",
    "　", "ぁ", "あ", "ぃ", "い", "ぅ", "う", "ぇ", "え", "ぉ", "お", "か", "が", "き", "ぎ", "く",
    "ぐ", "け", "げ", "こ", "ご", "さ", "ざ", "し", "じ", "す", "ず", "せ", "ぜ", "そ", "ぞ", "た",
    "だ", "ち", "ぢ", "っ", "つ", "づ", "て", "で", "と", "ど", "な", "に", "ぬ", "ね", "の", "は",
    "ば", "ぱ", "ひ", "び", "ぴ", "ふ", "ぶ", "ぷ", "へ", "べ", "ぺ", "ほ", "ぼ", "ぽ", "ま", "み",
    "む", "め", "も", "ゃ", "や", "ゅ", "ゆ", "ょ", "よ", "ら", "り", "る", "れ", "ろ", "ゎ", "わ",
    "ゐ", "ゑ", "を", "ん", "　", "　", "　", "ゝ", "ゞ", "ー", "。", "「", "」", "、", "・", "　",
];

// ひらがなコード -> 文字変換処理(リターン:文字(&str))
pub fn put_hiragana_char(code: u8) -> &'static str {

    // リターン情報(AC_HIRAGANA_TABLEの内容)
    AC_HIRAGANA_TABLE[code as usize]

}

const AC_KATAKANA_TABLE: [&str; 0x80] = [
    "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　",
    "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　",
    "　", "ァ", "ア", "ィ", "イ", "ゥ", "ウ", "ェ", "エ", "ォ", "オ", "カ", "ガ", "キ", "ギ", "ク",
    "グ", "ケ", "ゲ", "コ", "ゴ", "サ", "ザ", "シ", "ジ", "ス", "ズ", "セ", "ゼ", "ソ", "ゾ", "タ",
    "ダ", "チ", "ヂ", "ッ", "ツ", "ヅ", "テ", "デ", "ト", "ド", "ナ", "ニ", "ヌ", "ネ", "ノ", "ハ",
    "バ", "パ", "ヒ", "ビ", "ピ", "フ", "ブ", "プ", "ヘ", "ベ", "ペ", "ホ", "ボ", "ポ", "マ", "ミ",
    "ム", "メ", "モ", "ャ", "ヤ", "ュ", "ユ", "ョ", "ヨ", "ラ", "リ", "ル", "レ", "ロ", "ヮ", "ワ",
    "ヰ", "ヱ", "ヲ", "ン", "ヴ", "ヵ", "ヶ", "ヽ", "ヾ", "ー", "。", "「", "」", "、", "・", "　",
];

// カタカナコード -> 文字変換処理(リターン:文字(&str))
pub fn put_katakana_char(code: u8) -> &'static str {

    // リターン情報(AC_KATAKANA_TABLEの内容)
    AC_KATAKANA_TABLE[code as usize]

}

const AC_JIS_KATAKANA_TABLE: [&str; 0x80] = [
    "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　",
    "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　",
    "　", "。", "「", "」", "、", "・", "ヲ", "ァ", "ィ", "ゥ", "ェ", "ォ", "ャ", "ュ", "ョ", "ッ",
    "ー", "ア", "イ", "ウ", "エ", "オ", "カ", "キ", "ク", "ケ", "コ", "サ", "シ", "ス", "セ", "ソ",
    "タ", "チ", "ツ", "テ", "ト", "ナ", "ニ", "ヌ", "ネ", "ノ", "ハ", "ヒ", "フ", "ヘ", "ホ", "マ",
    "ミ", "ム", "メ", "モ", "ヤ", "ユ", "ヨ", "ラ", "リ", "ル", "レ", "ロ", "ワ", "ン", "゛", "゜",
    "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　",
    "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　", "　",
];

// JISカタカナコード -> 文字変換処理(リターン:文字(&str))
pub fn put_jis_katakana_char(code: u8) -> &'static str {

    // リターン情報(AC_JIS_KATAKANA_TABLEの内容)
    AC_JIS_KATAKANA_TABLE[code as usize]

}

// シンボルテーブル１
const ASZ_SYMBOLES_TABLE1_U8: [&[u8]; 0x25] = [
    &[0x5b, 0x48, 0x56, 0x5d],                                    // 0x7a50 "[HV]"
    &[0x5b, 0x53, 0x44, 0x5d],                                    // 0x7a51 "[SD]"
    &[0x5b, 0xef, 0xbc, 0xb0, 0x5d],                              // 0x7a52 "[Ｐ]"
    &[0x5b, 0xef, 0xbc, 0xb7, 0x5d],                              // 0x7a53 "[Ｗ]"
    &[0x5b, 0x4d, 0x56, 0x5d],                                    // 0x7a54 "[MV]"
    &[0x5b, 0xe6, 0x89, 0x8b, 0x5d],                              // 0x7a55 "[手]"
    &[0x5b, 0xe5, 0xad, 0x97, 0x5d],                              // 0x7a56 "[字]"
    &[0x5b, 0xe5, 0x8f, 0x8c, 0x5d],                              // 0x7a57 "[双]"
    &[0x5b, 0xe3, 0x83, 0x87, 0x5d],                              // 0x7a58 "[デ]"
    &[0x5b, 0xef, 0xbc, 0xb3, 0x5d],                              // 0x7a59 "[Ｓ]"
    &[0x5b, 0xe4, 0xba, 0x8c, 0x5d],                              // 0x7a5a "[二]"
    &[0x5b, 0xe5, 0xa4, 0x9a, 0x5d],                              // 0x7a5b "[多]"
    &[0x5b, 0xe8, 0xa7, 0xa3, 0x5d],                              // 0x7a5c "[解]"
    &[0x5b, 0x53, 0x53, 0x5d],                                    // 0x7a5d "[SS]"
    &[0x5b, 0xef, 0xbc, 0xa2, 0x5d],                              // 0x7a5e "[Ｂ]"
    &[0x5b, 0xef, 0xbc, 0xae, 0x5d],                              // 0x7a5f "[Ｎ]"
    &[0xe2, 0x96, 0xa0],                                          // 0x7a60 "■"
    &[0xe2, 0x97, 0x8f],                                          // 0x7a61 "●"
    &[0x5b, 0xe5, 0xa4, 0xa9, 0x5d],                              // 0x7a62 "[天]"
    &[0x5b, 0xe4, 0xba, 0xa4, 0x5d],                              // 0x7a63 "[交]"
    &[0x5b, 0xe6, 0x98, 0xa0, 0x5d],                              // 0x7a64 "[映]"
    &[0x5b, 0xe7, 0x84, 0xa1, 0x5d],                              // 0x7a65 "[無]"
    &[0x5b, 0xe6, 0x96, 0x99, 0x5d],                              // 0x7a66 "[料]"
    &[0x5b, 0xe5, 0xb9, 0xb4, 0xe9, 0xbd, 0xa2, 0xe5, 0x88, 0xb6, 0xe9, 0x99, 0x90, 0x5d],// 0x7a67 "[年齢制限]"
    &[0x5b, 0xe5, 0x89, 0x8d, 0x5d],                              // 0x7a68 "[前]"
    &[0x5b, 0xe5, 0xbe, 0x8c, 0x5d],                              // 0x7a69 "[後]"
    &[0x5b, 0xe5, 0x86, 0x8d, 0x5d],                              // 0x7a6a "[再]"
    &[0x5b, 0xe6, 0x96, 0xb0, 0x5d],                              // 0x7a6b "[新]"
    &[0x5b, 0xe5, 0x88, 0x9d, 0x5d],                              // 0x7a6c "[初]"
    &[0x5b, 0xe7, 0xb5, 0x82, 0x5d],                              // 0x7a6d "[終]"
    &[0x5b, 0xe7, 0x94, 0x9f, 0x5d],                              // 0x7a6e "[生]"
    &[0x5b, 0xe8, 0xb2, 0xa9, 0x5d],                              // 0x7a6f "[販]"
    &[0x5b, 0xe5, 0xa3, 0xb0, 0x5d],                              // 0x7a70 "[声]"
    &[0x5b, 0xe5, 0x90, 0xb9, 0x5d],                              // 0x7a71 "[吹]"
    &[0x5b, 0x50, 0x50, 0x56, 0x5d],                              // 0x7a72 "[PPV]"
    &[0x28, 0xe7, 0xa7, 0x98, 0x29],                              // 0x7a73 "(秘)"
    &[0xe3, 0x81, 0xbb, 0xe3, 0x81, 0x8b],                        // 0x7a74 "ほか"
];

// シンボルテーブル２
const ASZ_SYMBOLES_TABLE2_U8: [&[u8]; 0x5b] = [
    &[0xe2, 0x86, 0x92],                                          // 0x7c21 "→"
    &[0xe2, 0x86, 0x90],                                          // 0x7c22 "←"
    &[0xe2, 0x86, 0x91],                                          // 0x7c23 "↑"
    &[0xe2, 0x86, 0x93],                                          // 0x7c24 "↓"
    &[0xe2, 0x97, 0x8f],                                          // 0x7c25 "●"
    &[0xe2, 0x97, 0x8b],                                          // 0x7c26 "○"
    &[0xe5, 0xb9, 0xb4],                                          // 0x7c27 "年"
    &[0xe6, 0x9c, 0x88],                                          // 0x7c28 "月"
    &[0xe6, 0x97, 0xa5],                                          // 0x7c29 "日"
    &[0xe5, 0x86, 0x86],                                          // 0x7c2a "円"
    &[0xe3, 0x8e, 0xa1],                                          // 0x7c2b "㎡"
    &[0xe3, 0x8e, 0xa5],                                          // 0x7c2c "㎥"
    &[0xe3, 0x8e, 0x9d],                                          // 0x7c2d "㎝"
    &[0xe3, 0x8e, 0xa0],                                          // 0x7c2e "㎠"
    &[0xe3, 0x8e, 0xa4],                                          // 0x7c2f "㎤"
    &[0xef, 0xbc, 0x90, 0x2e],                                    // 0x7c30 "０."
    &[0xef, 0xbc, 0x91, 0x2e],                                    // 0x7c31 "１."
    &[0xef, 0xbc, 0x92, 0x2e],                                    // 0x7c32 "２."
    &[0xef, 0xbc, 0x93, 0x2e],                                    // 0x7c33 "３."
    &[0xef, 0xbc, 0x94, 0x2e],                                    // 0x7c34 "４."
    &[0xef, 0xbc, 0x95, 0x2e],                                    // 0x7c35 "５."
    &[0xef, 0xbc, 0x96, 0x2e],                                    // 0x7c36 "６."
    &[0xef, 0xbc, 0x97, 0x2e],                                    // 0x7c37 "７."
    &[0xef, 0xbc, 0x98, 0x2e],                                    // 0x7c38 "８."
    &[0xef, 0xbc, 0x99, 0x2e],                                    // 0x7c39 "９."
    &[0xe6, 0xb0, 0x8f],                                          // 0x7c3a "氏"
    &[0xe5, 0x89, 0xaf],                                          // 0x7c3b "副"
    &[0xe5, 0x85, 0x83],                                          // 0x7c3c "元"
    &[0xe6, 0x95, 0x85],                                          // 0x7c3d "故"
    &[0xe5, 0x89, 0x8d],                                          // 0x7c3e "前"
    &[0x5b, 0xe6, 0x96, 0xb0, 0x5d],                              // 0x7c3f "[新]"
    &[0xef, 0xbc, 0x90, 0x2c],                                    // 0x7c40 "０,"
    &[0xef, 0xbc, 0x91, 0x2c],                                    // 0x7c41 "１,"
    &[0xef, 0xbc, 0x92, 0x2c],                                    // 0x7c42 "２,"
    &[0xef, 0xbc, 0x93, 0x2c],                                    // 0x7c43 "３,"
    &[0xef, 0xbc, 0x94, 0x2c],                                    // 0x7c44 "４,"
    &[0xef, 0xbc, 0x95, 0x2c],                                    // 0x7c45 "５,"
    &[0xef, 0xbc, 0x96, 0x2c],                                    // 0x7c46 "６,"
    &[0xef, 0xbc, 0x97, 0x2c],                                    // 0x7c47 "７,"
    &[0xef, 0xbc, 0x98, 0x2c],                                    // 0x7c48 "８,"
    &[0xef, 0xbc, 0x99, 0x2c],                                    // 0x7c49 "９,"
    &[0x28, 0xe7, 0xa4, 0xbe, 0x29],                              // 0x7c4a "(社)"
    &[0x28, 0xe8, 0xb2, 0xa1, 0x29],                              // 0x7c4b "(財)"
    &[0x28, 0xe6, 0x9c, 0x89, 0x29],                              // 0x7c4c "(有)"
    &[0x28, 0xe6, 0xa0, 0xaa, 0x29],                              // 0x7c4d "(株)"
    &[0x28, 0xe4, 0xbb, 0xa3, 0x29],                              // 0x7c4e "(代)"
    &[0x28, 0xe5, 0x95, 0x8f, 0x29],                              // 0x7c4f "(問)"
    &[0xe2, 0x96, 0xb6],                                          // 0x7c50 "▶"
    &[0xe2, 0x97, 0x80],                                          // 0x7c51 "◀"
    &[0xe3, 0x80, 0x96],                                          // 0x7c52 "〖"
    &[0xe3, 0x80, 0x97],                                          // 0x7c53 "〗"
    &[0xe2, 0x9f, 0x90],                                          // 0x7c54 "⟐"
    &[0x5e, 0x32],                                                // 0x7c55 "^2"
    &[0x5e, 0x33],                                                // 0x7c56 "^3"
    &[0x28, 0x43, 0x44, 0x29],                                    // 0x7c57 "(CD)"
    &[0x28, 0x76, 0x6e, 0x29],                                    // 0x7c58 "(vn)"
    &[0x28, 0x6f, 0x62, 0x29],                                    // 0x7c59 "(ob)"
    &[0x28, 0x63, 0x62, 0x29],                                    // 0x7c5a "(cb)"
    &[0x28, 0x63, 0x65],                                          // 0x7c5b "(ce"
    &[0x6d, 0x62, 0x29],                                          // 0x7c5c "mb)"
    &[0x28, 0x68, 0x70, 0x29],                                    // 0x7c5d "(hp)"
    &[0x28, 0x62, 0x72, 0x29],                                    // 0x7c5e "(br)"
    &[0x28, 0x70, 0x29],                                          // 0x7c5f "(p)"
    &[0x28, 0x73, 0x29],                                          // 0x7c60 "(s)"
    &[0x28, 0x6d, 0x73, 0x29],                                    // 0x7c61 "(ms)"
    &[0x28, 0x74, 0x29],                                          // 0x7c62 "(t)"
    &[0x28, 0x62, 0x73, 0x29],                                    // 0x7c63 "(bs)"
    &[0x28, 0x62, 0x29],                                          // 0x7c64 "(b)"
    &[0x28, 0x74, 0x62, 0x29],                                    // 0x7c65 "(tb)"
    &[0x28, 0x74, 0x70, 0x29],                                    // 0x7c66 "(tp)"
    &[0x28, 0x64, 0x73, 0x29],                                    // 0x7c67 "(ds)"
    &[0x28, 0x61, 0x67, 0x29],                                    // 0x7c68 "(ag)"
    &[0x28, 0x65, 0x67, 0x29],                                    // 0x7c69 "(eg)"
    &[0x28, 0x76, 0x6f, 0x29],                                    // 0x7c6a "(vo)"
    &[0x28, 0x66, 0x6c, 0x29],                                    // 0x7c6b "(fl)"
    &[0x28, 0x6b, 0x65],                                          // 0x7c6c "(ke"
    &[0x79, 0x29],                                                // 0x7c6d "y)"
    &[0x28, 0x73, 0x61],                                          // 0x7c6e "(sa"
    &[0x78, 0x29],                                                // 0x7c6f "x)"
    &[0x28, 0x73, 0x79],                                          // 0x7c70 "(sy"
    &[0x6e, 0x29],                                                // 0x7c71 "n)"
    &[0x28, 0x6f, 0x72],                                          // 0x7c72 "(or"
    &[0x67, 0x29],                                                // 0x7c73 "g)"
    &[0x28, 0x70, 0x65],                                          // 0x7c74 "(pe"
    &[0x72, 0x29],                                                // 0x7c75 "r)"
    &[0x28, 0x52, 0x29],                                          // 0x7c76 "(R)"
    &[0x28, 0x43, 0x29],                                          // 0x7c77 "(C)"
    &[0x28, 0xe7, 0xae, 0x8f, 0x29],                              // 0x7c78 "(箏)"
    &[0x44, 0x4a],                                                // 0x7c79 "DJ"
    &[0x5b, 0xe6, 0xbc, 0x94, 0x5d],                              // 0x7c7a "[演]"
    &[0x46, 0x61, 0x78],                                          // 0x7c7b "Fax"
];

// シンボルテーブル３
const ASZ_SYMBOLES_TABLE3_U8: [&[u8]; 0x5b] = [
    &[0xe3, 0x88, 0xaa],                                          // 0x7d21 "㈪"
    &[0xe3, 0x88, 0xab],                                          // 0x7d22 "㈫"
    &[0xe3, 0x88, 0xac],                                          // 0x7d23 "㈬"
    &[0xe3, 0x88, 0xad],                                          // 0x7d24 "㈭"
    &[0xe3, 0x88, 0xae],                                          // 0x7d25 "㈮"
    &[0xe3, 0x88, 0xaf],                                          // 0x7d26 "㈯"
    &[0xe3, 0x88, 0xb0],                                          // 0x7d27 "㈰"
    &[0xe3, 0x88, 0xb7],                                          // 0x7d28 "㈷"
    &[0xe3, 0x8d, 0xbe],                                          // 0x7d29 "㍾"
    &[0xe3, 0x8d, 0xbd],                                          // 0x7d2a "㍽"
    &[0xe3, 0x8d, 0xbc],                                          // 0x7d2b "㍼"
    &[0xe3, 0x8d, 0xbb],                                          // 0x7d2c "㍻"
    &[0xe2, 0x84, 0x96],                                          // 0x7d2d "№"
    &[0xe2, 0x84, 0xa1],                                          // 0x7d2e "℡"
    &[0xe3, 0x80, 0xb6],                                          // 0x7d2f "〶"
    &[0xe2, 0x97, 0x8b],                                          // 0x7d30 "○"
    &[0xe3, 0x80, 0x94, 0xe6, 0x9c, 0xac, 0xe3, 0x80, 0x95],      // 0x7d31 "〔本〕"
    &[0xe3, 0x80, 0x94, 0xe4, 0xb8, 0x89, 0xe3, 0x80, 0x95],      // 0x7d32 "〔三〕"
    &[0xe3, 0x80, 0x94, 0xe4, 0xba, 0x8c, 0xe3, 0x80, 0x95],      // 0x7d33 "〔二〕"
    &[0xe3, 0x80, 0x94, 0xe5, 0xae, 0x89, 0xe3, 0x80, 0x95],      // 0x7d34 "〔安〕"
    &[0xe3, 0x80, 0x94, 0xe7, 0x82, 0xb9, 0xe3, 0x80, 0x95],      // 0x7d35 "〔点〕"
    &[0xe3, 0x80, 0x94, 0xe6, 0x89, 0x93, 0xe3, 0x80, 0x95],      // 0x7d36 "〔打〕"
    &[0xe3, 0x80, 0x94, 0xe7, 0x9b, 0x97, 0xe3, 0x80, 0x95],      // 0x7d37 "〔盗〕"
    &[0xe3, 0x80, 0x94, 0xe5, 0x8b, 0x9d, 0xe3, 0x80, 0x95],      // 0x7d38 "〔勝〕"
    &[0xe3, 0x80, 0x94, 0xe6, 0x95, 0x97, 0xe3, 0x80, 0x95],      // 0x7d39 "〔敗〕"
    &[0xe3, 0x80, 0x94, 0xef, 0xbc, 0xb3, 0xe3, 0x80, 0x95],      // 0x7d3a "〔Ｓ〕"
    &[0xef, 0xbc, 0xbb, 0xe6, 0x8a, 0x95, 0xef, 0xbc, 0xbd],      // 0x7d3b "［投］"
    &[0xef, 0xbc, 0xbb, 0xe6, 0x8d, 0x95, 0xef, 0xbc, 0xbd],      // 0x7d3c "［捕］"
    &[0xef, 0xbc, 0xbb, 0xe4, 0xb8, 0x80, 0xef, 0xbc, 0xbd],      // 0x7d3d "［一］"
    &[0xef, 0xbc, 0xbb, 0xe4, 0xba, 0x8c, 0xef, 0xbc, 0xbd],      // 0x7d3e "［二］"
    &[0xef, 0xbc, 0xbb, 0xe4, 0xb8, 0x89, 0xef, 0xbc, 0xbd],      // 0x7d3f "［三］"
    &[0xef, 0xbc, 0xbb, 0xe9, 0x81, 0x8a, 0xef, 0xbc, 0xbd],      // 0x7d40 "［遊］"
    &[0xef, 0xbc, 0xbb, 0xe5, 0xb7, 0xa6, 0xef, 0xbc, 0xbd],      // 0x7d41 "［左］"
    &[0xef, 0xbc, 0xbb, 0xe4, 0xb8, 0xad, 0xef, 0xbc, 0xbd],      // 0x7d42 "［中］"
    &[0xef, 0xbc, 0xbb, 0xe5, 0x8f, 0xb3, 0xef, 0xbc, 0xbd],      // 0x7d43 "［右］"
    &[0xef, 0xbc, 0xbb, 0xe6, 0x8c, 0x87, 0xef, 0xbc, 0xbd],      // 0x7d44 "［指］"
    &[0xef, 0xbc, 0xbb, 0xe8, 0xb5, 0xb0, 0xef, 0xbc, 0xbd],      // 0x7d45 "［走］"
    &[0xef, 0xbc, 0xbb, 0xe6, 0x89, 0x93, 0xef, 0xbc, 0xbd],      // 0x7d46 "［打］"
    &[0xe3, 0x8d, 0x91],                                          // 0x7d47 "㍑"
    &[0xe3, 0x8e, 0x8f],                                          // 0x7d48 "㎏"
    &[0xe3, 0x8e, 0x90],                                          // 0x7d49 "㎐"
    &[0x68, 0x61],                                                // 0x7d4a "ha"
    &[0xe3, 0x8e, 0x9e],                                          // 0x7d4b "㎞"
    &[0xe3, 0x8e, 0xa2],                                          // 0x7d4c "㎢"
    &[0xe3, 0x8d, 0xb1],                                          // 0x7d4d "㍱"
    &[0xe3, 0x83, 0xbb],                                          // 0x7d4e 未使用 "・"
    &[0xe3, 0x83, 0xbb],                                          // 0x7d4f 未使用 "・"
    &[0x31, 0x2f, 0x32],                                          // 0x7d50 "1/2"
    &[0x30, 0x2f, 0x33],                                          // 0x7d51 "0/3"
    &[0x31, 0x2f, 0x33],                                          // 0x7d52 "1/3"
    &[0x32, 0x2f, 0x33],                                          // 0x7d53 "2/3"
    &[0x31, 0x2f, 0x34],                                          // 0x7d54 "1/4"
    &[0x33, 0x2f, 0x34],                                          // 0x7d55 "3/4"
    &[0x31, 0x2f, 0x35],                                          // 0x7d56 "1/5"
    &[0x32, 0x2f, 0x35],                                          // 0x7d57 "2/5"
    &[0x33, 0x2f, 0x35],                                          // 0x7d58 "3/5"
    &[0x34, 0x2f, 0x35],                                          // 0x7d59 "4/5"
    &[0x31, 0x2f, 0x36],                                          // 0x7d5a "1/6"
    &[0x35, 0x2f, 0x36],                                          // 0x7d5b "5/6"
    &[0x31, 0x2f, 0x37],                                          // 0x7d5c "1/7"
    &[0x31, 0x2f, 0x38],                                          // 0x7d5d "1/8"
    &[0x31, 0x2f, 0x39],                                          // 0x7d5e "1/9"
    &[0x31, 0x2f, 0x31, 0x30],                                    // 0x7d5f "1/10"
    &[0xe2, 0x98, 0x80],                                          // 0x7d60 "☀"
    &[0xe2, 0x98, 0x81],                                          // 0x7d61 "☁"
    &[0xe2, 0x98, 0x82],                                          // 0x7d62 "☂"
    &[0xe2, 0x9b, 0x84],                                          // 0x7d63 "⛄"
    &[0xe2, 0x98, 0x96],                                          // 0x7d64 "☖"
    &[0xe2, 0x98, 0x97],                                          // 0x7d65 "☗"
    &[0xe2, 0x96, 0xbd],                                          // 0x7d66 "▽"
    &[0xe2, 0x96, 0xbc],                                          // 0x7d67 "▼"
    &[0xe2, 0x99, 0xa6],                                          // 0x7d68 "♦"
    &[0xe2, 0x99, 0xa5],                                          // 0x7d69 "♥"
    &[0xe2, 0x99, 0xa3],                                          // 0x7d6a "♣"
    &[0xe2, 0x99, 0xa0],                                          // 0x7d6b "♠"
    &[0xe2, 0x8c, 0xba],                                          // 0x7d6c "⌺"
    &[0xe2, 0xa6, 0xbf],                                          // 0x7d6d "⦿"
    &[0xe2, 0x80, 0xbc],                                          // 0x7d6e "‼"
    &[0xe2, 0x81, 0x89],                                          // 0x7d6f "⁉"
    &[0x28, 0xe6, 0x9b, 0x87, 0x2f, 0xe6, 0x99, 0xb4, 0x29],      // 0x7d70 "(曇/晴)"
    &[0xe2, 0x98, 0x94],                                          // 0x7d71 "☔"
    &[0x28, 0xe9, 0x9b, 0xa8, 0x29],                              // 0x7d72 "(雨)"
    &[0x28, 0xe9, 0x9b, 0xaa, 0x29],                              // 0x7d73 "(雪)"
    &[0x28, 0xe5, 0xa4, 0xa7, 0xe9, 0x9b, 0xaa, 0x29],            // 0x7d74 "(大雪)"
    &[0xe2, 0x9a, 0xa1],                                          // 0x7d75 "⚡"
    &[0x28, 0xe9, 0x9b, 0xb7, 0xe9, 0x9b, 0xa8, 0x29],            // 0x7d76 "(雷雨)"
    &[0xe2, 0x9b, 0x88],                                          // 0x7d77 "⛈"
    &[0xe2, 0x9a, 0x9e],                                          // 0x7d78 "⚞"
    &[0xe2, 0x9a, 0x9f],                                          // 0x7d79 "⚟"
    &[0xe2, 0x99, 0xac],                                          // 0x7d7a "♬"
    &[0xe2, 0x98, 0x8e],                                          // 0x7d7b "☎"
];

// シンボルテーブル４
const ASZ_SYMBOLES_TABLE4_U8: [&[u8]; 0x5d] = [
    &[0xe2, 0x85, 0xa0],                                          // 0x7e21 "Ⅰ"
    &[0xe2, 0x85, 0xa1],                                          // 0x7e22 "Ⅱ"
    &[0xe2, 0x85, 0xa2],                                          // 0x7e23 "Ⅲ"
    &[0xe2, 0x85, 0xa3],                                          // 0x7e24 "Ⅳ"
    &[0xe2, 0x85, 0xa4],                                          // 0x7e25 "Ⅴ"
    &[0xe2, 0x85, 0xa5],                                          // 0x7e26 "Ⅵ"
    &[0xe2, 0x85, 0xa6],                                          // 0x7e27 "Ⅶ"
    &[0xe2, 0x85, 0xa7],                                          // 0x7e28 "Ⅷ"
    &[0xe2, 0x85, 0xa8],                                          // 0x7e29 "Ⅸ"
    &[0xe2, 0x85, 0xa9],                                          // 0x7e2a "Ⅹ"
    &[0xe2, 0x85, 0xaa],                                          // 0x7e2b "Ⅺ"
    &[0xe2, 0x85, 0xab],                                          // 0x7e2c "Ⅻ"
    &[0xe2, 0x91, 0xb0],                                          // 0x7e2d "⑰"
    &[0xe2, 0x91, 0xb1],                                          // 0x7e2e "⑱"
    &[0xe2, 0x91, 0xb2],                                          // 0x7e2f "⑲"
    &[0xe2, 0x91, 0xb3],                                          // 0x7e30 "⑳"
    &[0xe2, 0x91, 0xb4],                                          // 0x7e31 "⑴"
    &[0xe2, 0x91, 0xb5],                                          // 0x7e32 "⑵"
    &[0xe2, 0x91, 0xb6],                                          // 0x7e33 "⑶"
    &[0xe2, 0x91, 0xb7],                                          // 0x7e34 "⑷"
    &[0xe2, 0x91, 0xb8],                                          // 0x7e35 "⑸"
    &[0xe2, 0x91, 0xb9],                                          // 0x7e36 "⑹"
    &[0xe2, 0x91, 0xba],                                          // 0x7e37 "⑺"
    &[0xe2, 0x91, 0xbb],                                          // 0x7e38 "⑻"
    &[0xe2, 0x91, 0xbc],                                          // 0x7e39 "⑼"
    &[0xe2, 0x91, 0xbd],                                          // 0x7e3a "⑽"
    &[0xe2, 0x91, 0xbe],                                          // 0x7e3b "⑾"
    &[0xe2, 0x91, 0xbf],                                          // 0x7e3c "⑿"
    &[0xe3, 0x89, 0x91],                                          // 0x7e3d "㉑"
    &[0xe3, 0x89, 0x92],                                          // 0x7e3e "㉒"
    &[0xe3, 0x89, 0x93],                                          // 0x7e3f "㉓"
    &[0xe3, 0x89, 0x94],                                          // 0x7e40 "㉔"
    &[0x28, 0x41, 0x29],                                          // 0x7e41 "(A)"
    &[0x28, 0x42, 0x29],                                          // 0x7e42 "(B)"
    &[0x28, 0x43, 0x29],                                          // 0x7e43 "(C)"
    &[0x28, 0x44, 0x29],                                          // 0x7e44 "(D)"
    &[0x28, 0x45, 0x29],                                          // 0x7e45 "(E)"
    &[0x28, 0x46, 0x29],                                          // 0x7e46 "(F)"
    &[0x28, 0x47, 0x29],                                          // 0x7e47 "(G)"
    &[0x28, 0x48, 0x29],                                          // 0x7e48 "(H)"
    &[0x28, 0x49, 0x29],                                          // 0x7e49 "(I)"
    &[0x28, 0x4a, 0x29],                                          // 0x7e4a "(J)"
    &[0x28, 0x4b, 0x29],                                          // 0x7e4b "(K)"
    &[0x28, 0x4c, 0x29],                                          // 0x7e4c "(L)"
    &[0x28, 0x4d, 0x29],                                          // 0x7e4d "(M)"
    &[0x28, 0x4e, 0x29],                                          // 0x7e4e "(N)"
    &[0x28, 0x4f, 0x29],                                          // 0x7e4f "(O)"
    &[0x28, 0x50, 0x29],                                          // 0x7e50 "(P)"
    &[0x28, 0x51, 0x29],                                          // 0x7e51 "(Q)"
    &[0x28, 0x52, 0x29],                                          // 0x7e52 "(R)"
    &[0x28, 0x53, 0x29],                                          // 0x7e53 "(S)"
    &[0x28, 0x54, 0x29],                                          // 0x7e54 "(T)"
    &[0x28, 0x55, 0x29],                                          // 0x7e55 "(U)"
    &[0x28, 0x56, 0x29],                                          // 0x7e56 "(V)"
    &[0x28, 0x57, 0x29],                                          // 0x7e57 "(W)"
    &[0x28, 0x58, 0x29],                                          // 0x7e58 "(X)"
    &[0x28, 0x59, 0x29],                                          // 0x7e59 "(Y)"
    &[0x28, 0x5a, 0x29],                                          // 0x7e5a "(Z)"
    &[0xe3, 0x89, 0x95],                                          // 0x7e5b "㉕"
    &[0xe3, 0x89, 0x96],                                          // 0x7e5c "㉖"
    &[0xe3, 0x89, 0x97],                                          // 0x7e5d "㉗"
    &[0xe3, 0x89, 0x98],                                          // 0x7e5e "㉘"
    &[0xe3, 0x89, 0x99],                                          // 0x7e5f "㉙"
    &[0xe3, 0x89, 0x9a],                                          // 0x7e60 "㉚"
    &[0xe2, 0x91, 0xa0],                                          // 0x7e61 "①"
    &[0xe2, 0x91, 0xa1],                                          // 0x7e62 "②"
    &[0xe2, 0x91, 0xa2],                                          // 0x7e63 "③"
    &[0xe2, 0x91, 0xa3],                                          // 0x7e64 "④"
    &[0xe2, 0x91, 0xa4],                                          // 0x7e65 "⑤"
    &[0xe2, 0x91, 0xa5],                                          // 0x7e66 "⑥"
    &[0xe2, 0x91, 0xa6],                                          // 0x7e67 "⑦"
    &[0xe2, 0x91, 0xa7],                                          // 0x7e68 "⑧"
    &[0xe2, 0x91, 0xa8],                                          // 0x7e69 "⑨"
    &[0xe2, 0x91, 0xa9],                                          // 0x7e6a "⑩"
    &[0xe2, 0x91, 0xaa],                                          // 0x7e6b "⑪"
    &[0xe2, 0x91, 0xab],                                          // 0x7e6c "⑫"
    &[0xe2, 0x91, 0xac],                                          // 0x7e6d "⑬"
    &[0xe2, 0x91, 0xad],                                          // 0x7e6e "⑭"
    &[0xe2, 0x91, 0xae],                                          // 0x7e6f "⑮"
    &[0xe2, 0x91, 0xaf],                                          // 0x7e70 "⑯"
    &[0xe2, 0x9d, 0xb6],                                          // 0x7e71 "❶"
    &[0xe2, 0x9d, 0xb7],                                          // 0x7e72 "❷"
    &[0xe2, 0x9d, 0xb8],                                          // 0x7e73 "❸"
    &[0xe2, 0x9d, 0xb9],                                          // 0x7e74 "❹"
    &[0xe2, 0x9d, 0xba],                                          // 0x7e75 "❺"
    &[0xe2, 0x9d, 0xbb],                                          // 0x7e76 "❻"
    &[0xe2, 0x9d, 0xbc],                                          // 0x7e77 "❼"
    &[0xe2, 0x9d, 0xbd],                                          // 0x7e78 "❽"
    &[0xe2, 0x9d, 0xbe],                                          // 0x7e79 "❾"
    &[0xe2, 0x9d, 0xbf],                                          // 0x7e7a "❿"
    &[0xe2, 0x93, 0xab],                                          // 0x7e7b "⓫"
    &[0xe2, 0x93, 0xac],                                          // 0x7e7c "⓬"
    &[0xe3, 0x89, 0x9b],                                          // 0x7e7d "㉛"
];

// シンボルテーブル５
const ASZ_SYMBOLES_TABLE5_U8: [&[u8]; 0x5e] = [
    &[0xe3, 0x90, 0x82],                                          // 0x7521 "㐂"
    &[0xf0, 0xa0, 0x85, 0x98],                                    // 0x7522 U+20158 "𠅘"
    &[0xe4, 0xbb, 0xbd],                                          // 0x7523 "份"
    &[0xe4, 0xbb, 0xbf],                                          // 0x7524 "仿"
    &[0xe4, 0xbe, 0x9a],                                          // 0x7525 "侚"
    &[0xe4, 0xbf, 0x89],                                          // 0x7526 "俉"
    &[0xe5, 0x82, 0x9c],                                          // 0x7527 "傜"
    &[0xe5, 0x84, 0x9e],                                          // 0x7528 "儞"
    &[0xe5, 0x86, 0xbc],                                          // 0x7529 "冼"
    &[0xe3, 0x94, 0x9f],                                          // 0x752a "㔟"
    &[0xe5, 0x8c, 0x87],                                          // 0x752b "匇"
    &[0xe5, 0x8d, 0xa1],                                          // 0x752c "卡"
    &[0xe5, 0x8d, 0xac],                                          // 0x752d "卬"
    &[0xe8, 0xa9, 0xb9],                                          // 0x752e "詹"
    &[0xf0, 0xa0, 0xae, 0xb7],                                    // 0x752f U+20BB7 "𠮷"
    &[0xe5, 0x91, 0x8d],                                          // 0x7530 "呍"
    &[0xe5, 0x92, 0x96],                                          // 0x7531 "咖"
    &[0xe5, 0x92, 0x9c],                                          // 0x7532 "咜"
    &[0xe5, 0x92, 0xa9],                                          // 0x7533 "咩"
    &[0xe5, 0x94, 0x8e],                                          // 0x7534 "唎"
    &[0xe5, 0x95, 0x8a],                                          // 0x7535 "啊"
    &[0xe5, 0x99, 0xb2],                                          // 0x7536 "噲"
    &[0xe5, 0x9b, 0xa4],                                          // 0x7537 "囤"
    &[0xe5, 0x9c, 0xb3],                                          // 0x7538 "圳"
    &[0xe5, 0x9c, 0xb4],                                          // 0x7539 "圴"
    &[0xef, 0xa8, 0x90],                                          // 0x753a "塚"
    &[0xe5, 0xa2, 0x80],                                          // 0x753b "墀"
    &[0xe5, 0xa7, 0xa4],                                          // 0x753c "姤"
    &[0xe5, 0xa8, 0xa3],                                          // 0x753d "娣"
    &[0xe5, 0xa9, 0x95],                                          // 0x753e "婕"
    &[0xe5, 0xaf, 0xac],                                          // 0x753f "寬"
    &[0xef, 0xa8, 0x91],                                          // 0x7540 "﨑"
    &[0xe3, 0x9f, 0xa2],                                          // 0x7541 "㟢"
    &[0xe5, 0xba, 0xac],                                          // 0x7542 "庬"
    &[0xe5, 0xbc, 0xb4],                                          // 0x7543 "弴"
    &[0xe5, 0xbd, 0x85],                                          // 0x7544 "彅"
    &[0xe5, 0xbe, 0xb7],                                          // 0x7545 "德"
    &[0xe6, 0x80, 0x97],                                          // 0x7546 "怗"
    &[0xe6, 0x81, 0xb5],                                          // 0x7547 "恵"
    &[0xe6, 0x84, 0xb0],                                          // 0x7548 "愰"
    &[0xe6, 0x98, 0xa4],                                          // 0x7549 "昤"
    &[0xe6, 0x9b, 0x88],                                          // 0x754a "曈"
    &[0xe6, 0x9b, 0x99],                                          // 0x754b "曙"
    &[0xe6, 0x9b, 0xba],                                          // 0x754c "曺"
    &[0xe6, 0x9b, 0xbb],                                          // 0x754d "曻"
    &[0xe6, 0xa1, 0x92],                                          // 0x754e "桒"
    &[0xe9, 0xbf, 0x84],                                          // 0x754f "鿄"
    &[0xe6, 0xa4, 0x91],                                          // 0x7550 "椑"
    &[0xe6, 0xa4, 0xbb],                                          // 0x7551 "椻"
    &[0xe6, 0xa9, 0x85],                                          // 0x7552 "橅"
    &[0xe6, 0xaa, 0x91],                                          // 0x7553 "檑"
    &[0xe6, 0xab, 0x9b],                                          // 0x7554 "櫛"
    &[0xf0,0xa3, 0x8f, 0x8c],                                     // 0x7555 U+233CC "𣏌"
    &[0xf0,0xa3, 0x8f, 0xbe],                                     // 0x7556 U+233FE "𣏾"
    &[0xf0,0xa3, 0x97, 0xf4],                                     // 0x7557 U+235C4 "𣗄"
    &[0xe6, 0xaf, 0xb1],                                          // 0x7558 "毱"
    &[0xe6, 0xb3, 0xa0],                                          // 0x7559 "泠"
    &[0xe6, 0xb4, 0xae],                                          // 0x755a "洮"
    &[0xef, 0xa9, 0x85],                                          // 0x755b "海"
    &[0xe6, 0xb6, 0xbf],                                          // 0x755c "涿"
    &[0xe6, 0xb7, 0x8a],                                          // 0x755d "淊"
    &[0xe6, 0xb7, 0xb8],                                          // 0x755e "淸"
    &[0xef, 0xa9, 0x86],                                          // 0x755f "渚"
    &[0xe6, 0xbd, 0x9e],                                          // 0x7560 "潞"
    &[0xe6, 0xbf, 0xb9],                                          // 0x7561 "濹"
    &[0xe7, 0x81, 0xa4],                                          // 0x7562 "灤"
    &[0xef, 0xa9, 0xac],                                          // 0x7563 "𤋮"
    &[0xf0,0xa4, 0x8b, 0xae],                                     // 0x7564 U+242EE "𤋮"
    &[0xe7, 0x85, 0x87],                                          // 0x7565 "煇"
    &[0xe7, 0x87, 0x81],                                          // 0x7566 "燁"
    &[0xe7, 0x88, 0x80],                                          // 0x7567 "爀"
    &[0xe7, 0x8e, 0x9f],                                          // 0x7568 "玟"
    &[0xe7, 0x8e, 0xa8],                                          // 0x7569 "玨"
    &[0xe7, 0x8f, 0x89],                                          // 0x756a "珉"
    &[0xe7, 0x8f, 0x96],                                          // 0x756b "珖"
    &[0xe7, 0x90, 0x9b],                                          // 0x756c "琛"
    &[0xe7, 0x90, 0xa1],                                          // 0x756d "琡"
    &[0xef, 0xa9, 0x8a],                                          // 0x756e "琢"
    &[0xe7, 0x90, 0xa6],                                          // 0x756f "琦"
    &[0xe7, 0x90, 0xaa],                                          // 0x7570 "琪"
    &[0xe7, 0x90, 0xac],                                          // 0x7571 "琬"
    &[0xe7, 0x90, 0xb9],                                          // 0x7572 "琹"
    &[0xe7, 0x91, 0x8b],                                          // 0x7573 "瑋"
    &[0xe3, 0xbb, 0x9a],                                          // 0x7574 "㻚"
    &[0xe7, 0x95, 0xb5],                                          // 0x7575 "畵"
    &[0xe7, 0x96, 0x81],                                          // 0x7576 "疁"
    &[0xe7, 0x9d, 0xb2],                                          // 0x7577 "睲"
    &[0xe4, 0x82, 0x93],                                          // 0x7578 "䂓"
    &[0xe7, 0xa3, 0x88],                                          // 0x7579 "磈"
    &[0xe7, 0xa3, 0xa0],                                          // 0x757a "磠"
    &[0xe7, 0xa5, 0x87],                                          // 0x757b "祇"
    &[0xe7, 0xa6, 0xae],                                          // 0x757c "禮"
    &[0xe9, 0xbf, 0x86],                                          // 0x757d "鿆"
    &[0xe4, 0x84, 0x83],                                          // 0x757e "䄃"
];

// シンボルテーブル６
const ASZ_SYMBOLES_TABLE6_U8: [&[u8]; 0x2b] = [
    &[0xe9, 0xbf, 0x85],                                          // 0x7621 "鿅"
    &[0xe7, 0xa7, 0x9a],                                          // 0x7622 "秚"
    &[0xe7, 0xa8, 0x9e],                                          // 0x7623 "稞"
    &[0xe7, 0xad, 0xbf],                                          // 0x7624 "筿"
    &[0xe7, 0xb0, 0xb1],                                          // 0x7625 "簱"
    &[0xe4, 0x89, 0xa4],                                          // 0x7626 "䉤"
    &[0xe7, 0xb6, 0x8b],                                          // 0x7627 "綋"
    &[0xe7, 0xbe, 0xa1],                                          // 0x7628 "羡"
    &[0xe8, 0x84, 0x98],                                          // 0x7629 "脘"
    &[0xe8, 0x84, 0xba],                                          // 0x762a "脺"
    &[0xef, 0xa9, 0xad],                                          // 0x762b "舘"
    &[0xe8, 0x8a, 0xae],                                          // 0x762c "芮"
    &[0xe8, 0x91, 0x9b],                                          // 0x762d "葛"
    &[0xe8, 0x93, 0x9c],                                          // 0x762e "蓜"
    &[0xe8, 0x93, 0xac],                                          // 0x762f "蓬"
    &[0xe8, 0x95, 0x99],                                          // 0x7630 "蕙"
    &[0xe8, 0x97, 0x8e],                                          // 0x7631 "藎"
    &[0xe8, 0x9d, 0x95],                                          // 0x7632 "蝕"
    &[0xe8, 0x9f, 0xac],                                          // 0x7633 "蟬"
    &[0xe8, 0xa0, 0x8b],                                          // 0x7634 "蠋"
    &[0xe8, 0xa3, 0xb5],                                          // 0x7635 "裵"
    &[0xe8, 0xa7, 0x92],                                          // 0x7636 "角"
    &[0xe8, 0xab, 0xb6],                                          // 0x7637 "諶"
    &[0xe8, 0xb7, 0x8e],                                          // 0x7638 "跎"
    &[0xe8, 0xbe, 0xbb],                                          // 0x7639 "辻"
    &[0xe8, 0xbf, 0xb6],                                          // 0x763a "迶"
    &[0xe9, 0x83, 0x9d],                                          // 0x763b "郝"
    &[0xe9, 0x84, 0xa7],                                          // 0x763c "鄧"
    &[0xe9, 0x84, 0xad],                                          // 0x763d "鄭"
    &[0xe9, 0x86, 0xb2],                                          // 0x763e "醲"
    &[0xe9, 0x88, 0xb3],                                          // 0x763f "鈳"
    &[0xe9, 0x8a, 0x88],                                          // 0x7640 "銈"
    &[0xe9, 0x8c, 0xa1],                                          // 0x7641 "錡"
    &[0xe9, 0x8d, 0x88],                                          // 0x7642 "鍈"
    &[0xe9, 0x96, 0x92],                                          // 0x7643 "閒"
    &[0xe9, 0x9b, 0x9e],                                          // 0x7644 "雞"
    &[0xe9, 0xa4, 0x83],                                          // 0x7645 "餃"
    &[0xe9, 0xa5, 0x80],                                          // 0x7646 "饀"
    &[0xe9, 0xab, 0x99],                                          // 0x7647 "髙"
    &[0xe9, 0xaf, 0x96],                                          // 0x7648 "鯖"
    &[0xe9, 0xb7, 0x97],                                          // 0x7649 "鷗"
    &[0xe9, 0xba, 0xb4],                                          // 0x764a "麴"
    &[0xe9, 0xba, 0xb5],                                          // 0x764b "麵"
];

// シンボルテーブル文字変換処理(リターン:文字(&str))
pub fn put_symbols_char(code: u32) -> &'static str {

    // シンボルテーブル１の処理
    if code >= 0x7a50 && code <= 0x7a74 {

        // 文字コード -> 文字変換
        let utf8_str: &str = std::str::from_utf8(&ASZ_SYMBOLES_TABLE1_U8[code as usize - 0x7a50]).unwrap();

        utf8_str

    }
    // シンボルテーブル２の処理
    else if code >= 0x7c21 && code <= 0x7c7b {

        // 文字コード -> 文字変換
        let utf8_str: &str = std::str::from_utf8(&ASZ_SYMBOLES_TABLE2_U8[code as usize - 0x7c21]).unwrap();

        utf8_str

    }
    // シンボルテーブル３の処理
    else if code >= 0x7d21 && code <= 0x7d7b {

        // 文字コード -> 文字変換
        let utf8_str: &str = std::str::from_utf8(&ASZ_SYMBOLES_TABLE3_U8[code as usize - 0x7d21]).unwrap();

        // リターン情報
        utf8_str

    }
    // シンボルテーブル４の処理
    else if code >= 0x7e21 && code <= 0x7e7d {

        // 文字コード -> 文字変換
        let utf8_str: &str = std::str::from_utf8(&ASZ_SYMBOLES_TABLE4_U8[code as usize - 0x7e21]).unwrap();

        // リターン情報
        utf8_str

    }
    // シンボルテーブル５の処理
    else if code >= 0x7521 && code <= 0x757e {

        // 文字コード -> 文字変換
        let utf8_str: &str = std::str::from_utf8(&ASZ_SYMBOLES_TABLE5_U8[code as usize - 0x7521]).unwrap();

        // リターン情報
        utf8_str

    }
    // シンボルテーブル６の処理
    else if code >= 0x7621 && code <= 0x764b {

        // 文字コード -> 文字変換
        let utf8_str: &str = std::str::from_utf8(&ASZ_SYMBOLES_TABLE6_U8[code as usize - 0x7621]).unwrap();

        // リターン情報
        utf8_str

    }
    else {

        // リターン情報(デフォルトは"・")
        "・"

    }
}

// シングルバイト文字の処理(リターン:文字(String))
fn single_byte(g_code: u32, code: u8) -> String {

    match g_code {

        CODE_ALPHANUMERIC | CODE_PROP_ALPHANUMERIC => {

            if STR_SIZE.load(Acquire) == STR_MEDIUM {

                // ミディアム文字の場合
                // リターン情報(ASCII -> UTF-8変換)
                std::str::from_utf8(&[code as u8].clone()).unwrap().to_string()

            }
            else {

                // ミディアム文字以外の場合
                // リターン情報(AC_ALPHANUMERIC_TABLEの内容)
                put_alphanumeric_char(code).to_string()

            }

        },
        CODE_HIRAGANA | CODE_PROP_HIRAGANA => {

            // リターン情報(AC_HIRAGANA_TABLEの内容)
            put_hiragana_char(code).to_string()

        },
        CODE_PROP_KATAKANA | CODE_KATAKANA => {

            // リターン情報(AC_KATAKANA_TABLEの内容)
            put_katakana_char(code).to_string()

        },
        CODE_JIS_X0201_KATAKANA => {

            // リターン情報(AC_JIS_KATAKANA_TABLEの内容)
            put_jis_katakana_char(code).to_string()

        },
        _ => {

            // リターン情報
            "".to_string()

        },
    }

}

// グラフィックセットページ情報の設定処理(リターン:処理状況(bool))
fn designation_set_graphic(g_num: usize, code: u8) -> bool {

    match code {
        0x42 => {  // Kanji

            G_PAGE[g_num].store(CODE_KANJI.try_into().unwrap(), Release);
            true

        },
        0x4a => {  // Alphanumeric

            G_PAGE[g_num].store(CODE_ALPHANUMERIC.try_into().unwrap(), Release);
            true

        },
        0x30 => {  // Hiragana

            G_PAGE[g_num].store(CODE_HIRAGANA.try_into().unwrap(), Release);
            true
                
        },
        0x31 => {  // Katakana

            G_PAGE[g_num].store(CODE_KATAKANA.try_into().unwrap(), Release);
            true
                
        },
        0x32 => {  // Mosaic A

            G_PAGE[g_num].store(CODE_MOSAIC_A.try_into().unwrap(), Release);
            true
                
        },
        0x33 => {  // Mosaic B

            G_PAGE[g_num].store(CODE_MOSAIC_B.try_into().unwrap(), Release);
            true
                
        },
        0x34 => {  // Mosaic C

            G_PAGE[g_num].store(CODE_MOSAIC_C.try_into().unwrap(), Release);
            true
                
        },
        0x35 => {  // Mosaic D

            G_PAGE[g_num].store(CODE_MOSAIC_D.try_into().unwrap(), Release);
            true
                
        },
        0x36 => {  // Proportional Alphanumeric

            G_PAGE[g_num].store(CODE_PROP_ALPHANUMERIC.try_into().unwrap(), Release);
            true

        },
        0x37 => {  // Proportional Hiragana

            G_PAGE[g_num].store(CODE_PROP_HIRAGANA.try_into().unwrap(), Release);
            true

        },
        0x38 => {  // Proportional Katakana

            G_PAGE[g_num].store(CODE_PROP_KATAKANA.try_into().unwrap(), Release);
            true
                
        },
        0x49 => {  // JIS X 0201 Katakana

            G_PAGE[g_num].store(CODE_JIS_X0201_KATAKANA.try_into().unwrap(), Release);
            true
                
        },
        0x39 => {  // JIS compatible Kanji Plane 1

            G_PAGE[g_num].store(CODE_JIS_KANJI_PLANE_1.try_into().unwrap(), Release);
            true
                
        },
        0x3a => { // JIS compatible Kanji Plane 2

            G_PAGE[g_num].store(CODE_JIS_KANJI_PLANE_2.try_into().unwrap(), Release);
            true
                
        },
        0x3b => {  // Additional symbols

            G_PAGE[g_num].store(CODE_ADDITIONAL_SYMBOLS.try_into().unwrap(), Release);
            true
                
        },
        _ => {  // 不明なグラフィックセット

            false

        },
    }
}

// グラフィックセットDRCS情報の設定処理(リターン:処理状況(bool))
fn designation_set_drcsgraphic(g_num: usize, code: u8) -> bool {


    match code {
        0x40 => {  // DRCS-0

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true

        },
        0x41 => {  // DRCS-1

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true
                
        },
        0x42 => {  // DRCS-2

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true

        },
        0x43 => {  // DRCS-3

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true
                
        },
        0x44 => {  // DRCS-4

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true
                
        },
        0x45 => {  // DRCS-5

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true

        },
        0x46 => {  // DRCS-6

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true
                
        },
        0x47 => {  // DRCS-7

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true
                
        },
        0x48 => {  // DRCS-8

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true

        },
        0x49 => {  // DRCS-9

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true
                
        },
        0x4a => {  // DRCS-10

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true
                
        },
        0x4b => {  // DRCS-11

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true
                
        },
        0x4c => {  // DRCS-12

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true
                
        },
        0x4d => {  // DRCS-13

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true
                
        },
        0x4e => {  // DRCS-14

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true
                
        },
        0x4f => {  // DRCS-15

            G_PAGE[g_num].store(CODE_UNKNOWN.try_into().unwrap(), Release);
            true
                
        },
        _ => {  // 不明なグラフィックセット

            false

        },
    }

}

// エスケープシーケンスの処理(リターン:処理バイト数)
fn escape_control(data: &[u8], index: usize) -> i32 {

    let mut _len: i32 = 0;

    if data.len() >= index {
        // GLページの処理
        if data[index + 1] == 0x24 && data.len() > index + 2 {

            if data[index + 2] >= 0x28 && data[index + 2] <= 0x2b && data.len() > index + 3 {

                if data[index + 3] == 0x20 && data.len() > index + 4 {

                    // グラフィックセットDRCSの設定シーケンス
                    match data[index + 2] {

                        0x28 => {

                            designation_set_drcsgraphic(0, data[index + 4]);
                            _len = 5;

                        },
                        0x29 => {

                            designation_set_drcsgraphic(1, data[index + 4]);
                            _len = 5;

                        },
                        0x2a => {

                            designation_set_drcsgraphic(2, data[index + 4]);
                            _len = 5;

                        },
                        0x2b => {

                            designation_set_drcsgraphic(3, data[index + 4]);
                            _len = 5;

                        },
                        _ => {

                            _len = 1;

                        },
                    }
                }
                else {
                    // グラフィックセットページの設定シーケンス
                    match data[index + 2] {
                        0x29 => {

                            designation_set_graphic(1,  data[index + 3]);
                            _len = 4;

                        },
                        0x2a => {

                            designation_set_graphic(2,  data[index + 3]);
                            _len = 4;

                        },
                        0x2b => {

                            designation_set_graphic(3,  data[index + 3]);
                            _len = 4;

                        },
                        _ => {

                            _len = 1;

                        },
                    }
                }
            }
            else {

                // グラフィックセットページの解除シーケンス
                designation_set_graphic(0, data[index + 2]);
                _len = 3;

            }
        }
        else if data[index + 1] >= 0x28 && data[index + 1] <= 0x2b && data.len() > index + 2 {

            if data[index + 2] == 0x20 && data.len() > index + 3 {

                // グラフィックセットDRCSの設定シーケンス
                match data[index + 1] {
                    0x28 => {

                        designation_set_drcsgraphic(0, data[index + 3]);
                        _len = 4;

                    },
                    0x29 => {

                        designation_set_drcsgraphic(1, data[index + 3]);
                        _len = 4;

                    },
                    0x2a => {

                        designation_set_drcsgraphic(2, data[index + 3]);
                        _len = 4;

                    },
                    0x2b => {

                        designation_set_drcsgraphic(3, data[index + 3]);
                        _len = 4;

                    },
                    _ => {

                        _len = 1;

                    },
                }

            }
            else {

                // グラフィックセットページの設定シーケンス
                match data[index + 1] {
                    0x28 => {

                        designation_set_graphic(0, data[index + 2]);
                        _len = 3;

                    },
                    0x29 => {

                        designation_set_graphic(1, data[index + 2]);
                        _len = 3;

                    },
                    0x2a => {

                        designation_set_graphic(2, data[index + 2]);
                        _len = 3;

                    },
                    0x2b => {

                        designation_set_graphic(3, data[index + 2]);
                        _len = 3;

                    },
                    _ => {

                        _len = 1;

                    },
                }

            }
        }
        else {

            _len = 1;

        };
    }
    else {
        _len = 1;
    };

    _len
}

// 制御コード文字の処理(リターン:処理バイト数、文字(String))
fn invocation_set_graphic(data: &[u8], index: usize) -> (i32, String) {

    #[allow(unused_assignments)]
    let mut len: i32 = 0;
    let mut _ret_str: String = "".to_string();

    let gl: u32 = G_PAGE[GL_NUM.load(Acquire) as usize].load(Acquire);
    let gr: u32 = G_PAGE[GR_NUM.load(Acquire) as usize].load(Acquire);

    // GLの処理
    #[allow(unused_comparisons)]
    if data[index] >= 0x00 && data[index] <= 0x20 {

        match data[index] {
            0x0f => { // LS0

                GL_NUM.store(0.try_into().unwrap(), Release);
                len = 1;

            },
            0x0e => { // LS1

                GL_NUM.store(1.try_into().unwrap(), Release);
                len = 1;

            },
            0x0d => { // LF

                _ret_str = String::from("");
                len = 1;

            },
            0x1b => { // ESC

                // 後続データがある場合にエスケープコード処理
                if data.len() > index + 1 {
                    match data[index + 1] {
                        0x6e => { // LS2

                            GL_NUM.store(2.try_into().unwrap(), Release);
                            len = 2;

                        },
                        0x6f => { // LS3

                            GL_NUM.store(3.try_into().unwrap(), Release);
                            len = 2;

                        },
                        0x7c => { // LS3R

                            GR_NUM.store(3.try_into().unwrap(), Release);
                            len = 2;

                        },
                        0x7d => { // LS2R

                            GR_NUM.store(2.try_into().unwrap(), Release);
                            len = 2;

                        },
                        0x7e => { // LS1R

                            GR_NUM.store(1.try_into().unwrap(), Release);
                            len = 2;

                        },
                        _ => {

                            len = escape_control(&data, index);

                        },
                    };
                }
                else {
                    len = 1;
                };
            },
            0x19 => { // SS2

                // GLコードページの退避
                let ss2 = GL_NUM.load(Acquire);
                GL_NUM.store(2.try_into().unwrap(), Release);

                // 後続データがある場合にarib_parse処理の再呼び出し
                if data.len() > index + 1 {

                    (len, _ret_str) = arib_parse(&data, index + 1);

                };

                // GLコードページの復旧
                GL_NUM.store(ss2.try_into().unwrap(), Release);

                len += 1;

            },
            0x1d => { // SS3

                // GLコードページの退避
                let ss3 = GL_NUM.load(Acquire);
                GL_NUM.store(3.try_into().unwrap(), Release);

                // 後続データがある場合にarib_parse処理の再呼び出し
                if data.len() > index + 1 {

                    (len, _ret_str) = arib_parse(&data, index + 1);

                };

                // GLコードページの復旧
                GL_NUM.store(ss3.try_into().unwrap(), Release);

                len += 1;

            },
            0x20 => { // スペース
                // 半角スペース
                if STR_SIZE.load(Acquire) == STR_MEDIUM {

                    _ret_str = String::from(" ");

                }
                // 全角スペース
                else if STR_SIZE.load(Acquire) == STR_NORMAL {

                    _ret_str = String::from("　");

                };

                len = 1;

            },
            _ => { // 上記以外は文字位置を進めるだけ

                // 文字サイズ大
                if AB_CHAR_SIZE_TABLE[gl as usize] == true {

                    len = 2;

                }
                // 文字サイズ小
                else {

                    len = 1;

                };

            },
        };

    }
    // GRの処理
    else {

        match data[index] {
            0x89 => { // MSZ

                STR_SIZE.store(STR_MEDIUM.try_into().unwrap(), Release);
                len = 1;

            },
            0x8a => { // NSZ

                STR_SIZE.store(STR_NORMAL.try_into().unwrap(), Release);
                len = 1;

            },
            0xa0 => { // スペース

                // 半角スペース
                if STR_SIZE.load(Acquire) == STR_MEDIUM {

                    _ret_str = String::from(" ");

                }
                // 全角スペース
                else if STR_SIZE.load(Acquire) == STR_NORMAL {

                    _ret_str = String::from("　");

                };

                len = 1;

            },
            _ => { // 上記以外は文字位置を進めるだけ
                // 文字サイズ大
                if AB_CHAR_SIZE_TABLE[gr as usize] == true {

                    len = 2;

                }
                // 文字サイズ小
                else {

                    len = 1;

                };
            },
        };
    };

    // リターン情報
    (len, _ret_str)

}

// 文字コード変換処理(リターン情報：文字長、文字(String))
fn arib_parse(data: &[u8], index: usize) -> (i32, String) {

    #[allow(unused_assignments)]
    let mut len: i32 = 0;
    let mut ret_str: String = String::new();

    let gl: u32 = G_PAGE[GL_NUM.load(Acquire) as usize].load(Acquire);
    let gr: u32 = G_PAGE[GR_NUM.load(Acquire) as usize].load(Acquire);

    // GL処理
    if data[index] >= 0x21 && data[index] <= 0x7e {

        // ２バイト文字
        if AB_CHAR_SIZE_TABLE[gl as usize] == true {

            if data.len() - index > 1 {

                // ２バイトコード処理
                ret_str = multi_byte(gl, data[index], data[index + 1]);
                len = 2;

            }
            else {

                len = 1;

            };
        }
        // １バイト文字
        else {

            // １バイトコード処理
            ret_str = single_byte(gl, data[index]);

            len = 1;

        };
    }
    // GR処理
    else if data[index] >= 0xa1 && data[index] <= 0xfe {

        // ２バイト文字
        if AB_CHAR_SIZE_TABLE[gr as usize] == true {

            if data.len() - index  > 1 {

                // ２バイトコード処理
                ret_str = multi_byte(gr, data[index] & 0x7f, data[index + 1] & 0x7f);

                len = 2;

            }
            else {

                len = 1;

            }
        }
        // １バイト文字
        else {

            // １バイトコード処理
            ret_str = single_byte(gr, data[index] & 0x7f);

            len = 1;

        };
    }
    // 制御コード
    else {

        //ret_str = "".to_string();
        (len, ret_str) = invocation_set_graphic(&data, index);

    };

    // リターン情報
    (len, ret_str)

}

// 文字コード -> 文字への変換処理(リターン:文字列長、文字列(String))
pub fn arib_to_string(data: &[u8], length: i32) -> (i32, String) {

    let mut _len: i32 = 0;
    let mut ret_data: String = String::new();
    let mut _ret_str: String = String::new();
    let mut index: usize = 0;
    let mut loop_len: i32 = length;

    // エスケープカウンター初期化
    ESC_SEQ_COUNT.store(0.try_into().unwrap(), Release);

    // グラフィックセットページ情報初期化
    G_PAGE[0].store(CODE_KANJI.try_into().unwrap(), Release);
    G_PAGE[1].store(CODE_ALPHANUMERIC.try_into().unwrap(), Release);
    G_PAGE[2].store(CODE_HIRAGANA.try_into().unwrap(), Release);
    G_PAGE[3].store(CODE_KATAKANA.try_into().unwrap(), Release);

    // GL.GR情報の初期化
    GL_NUM.store(0.try_into().unwrap(), Release);
    GR_NUM.store(2.try_into().unwrap(), Release);

    // 文字サイズ情報の初期化
    STR_SIZE.store(STR_NORMAL.try_into().unwrap(), Release);

    // データが無くなるまでループ
    while loop_len > 0 {

        // 文字コード変換処理呼び出し
        (_len, _ret_str) = arib_parse(&data, index);

        // リターン文字列作成
        if _len > 0 {
            ret_data.push_str(&_ret_str);
        };

        // カウンター更新
        index += _len as usize;
        loop_len -= _len;

    };

    // リターン情報
    (ret_data.len().try_into().unwrap(), ret_data)

}
