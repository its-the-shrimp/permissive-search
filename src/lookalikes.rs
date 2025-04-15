/// Returns `(row, col)`
const fn find_char<const N_ROWS: usize, const N_COLS: usize>(
    ch: char,
    layout: &[[char; N_COLS]; N_ROWS],
) -> Option<(usize, usize)> {
    let mut row = 0;
    while row < N_ROWS {
        let mut col = 0;
        while col < N_COLS {
            if layout[row][col] == ch {
                return Some((row, col));
            }
            col += 1;
        }
        row += 1;
    }

    None
}

/// Returns all characters that `ch` could've been a misclick of.
///
/// E.g. if the user typed in `a`, it could mean that they meant `a`, or (assuming their keybaord
/// is in the QWERTY layout) they've misclicked one of the following: `q`, `w`, `s`, `x`, `z`
pub fn qwerty_misclicks(ch: char) -> impl Iterator<Item = char> + Clone {
    static LAYOUT: [[char; 10]; 4] = [
        ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0'],
        ['q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p'],
        ['a', 's', 'd', 'f', 'g', 'h', 'j', 'k', 'l', ';'],
        ['z', 'x', 'c', 'v', 'b', 'n', 'm', ',', '.', '/'],
    ];
    static SHIFTED_LAYOUT: [[char; 10]; 4] = [
        ['!', '@', '#', '$', '%', '^', '&', '*', '(', ')'],
        ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'],
        ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L', ':'],
        ['Z', 'X', 'C', 'V', 'B', 'N', 'M', '<', '>', '?'],
    ];

    /// The number of ASCII characters that are printable & typeable
    const N_TYPEABLES: usize = (b'~' - b' ') as usize;

    /// The maximum number of misclicks this iterator can produce
    const N_MISCLICKS: usize = 17;

    static MISCLICKS: [[char; N_MISCLICKS]; N_TYPEABLES] = {
        const N_ROWS: usize = LAYOUT.len();
        const N_COLS: usize = LAYOUT[0].len();
        assert!(size_of_val(&LAYOUT) == size_of_val(&SHIFTED_LAYOUT));

        let mut res = [['\0'; 17]; N_TYPEABLES];
        let mut next_byte = b' ';
        while next_byte <= b'~' {
            let ch = next_byte as char;

            // Assembling the set
            let byte = next_byte;
            next_byte += 1;
            let (row, col, toggled) = if let Some((row, col)) = find_char(ch, &LAYOUT) {
                (row, col, SHIFTED_LAYOUT[row][col])
            } else if let Some((row, col)) = find_char(ch, &SHIFTED_LAYOUT) {
                (row, col, LAYOUT[row][col])
            } else {
                continue;
            };
            let (toggled_p, set) = res[(byte - b' ') as usize].split_first_mut().unwrap();
            *toggled_p = toggled;

            if row > 0 {
                if col > 0 {
                    set[0] = LAYOUT[row - 1][col - 1];
                    set[8] = SHIFTED_LAYOUT[row - 1][col - 1];
                }
                set[1] = LAYOUT[row - 1][col];
                set[9] = SHIFTED_LAYOUT[row - 1][col];
                if col < N_COLS - 1 {
                    set[2] = LAYOUT[row - 1][col + 1];
                    set[10] = SHIFTED_LAYOUT[row - 1][col + 1];
                }
            }
            if col > 0 {
                set[3] = LAYOUT[row][col - 1];
                set[11] = SHIFTED_LAYOUT[row][col - 1];
            }
            if col < N_COLS - 1 {
                set[4] = LAYOUT[row][col + 1];
                set[12] = SHIFTED_LAYOUT[row][col + 1];
            }
            if row < N_ROWS - 1 {
                if col > 0 {
                    set[5] = LAYOUT[row + 1][col - 1];
                    set[13] = SHIFTED_LAYOUT[row + 1][col - 1];
                }
                set[6] = LAYOUT[row + 1][col];
                set[14] = SHIFTED_LAYOUT[row + 1][col];
                if col < N_COLS - 1 {
                    set[7] = LAYOUT[row + 1][col + 1];
                    set[15] = SHIFTED_LAYOUT[row + 1][col + 1];
                }
            }

            // Moving all NULs to the end to enable short-curcuiting on the first NUL
            let mut shift = 0;
            let mut i = 0;
            while i < 16 {
                if set[i] == '\0' {
                    shift += 1;
                } else {
                    set.swap(i, i - shift);
                }
                i += 1;
            }
        }

        res
    };

    u32::from(ch)
        .checked_sub(b' '.into())
        .and_then(|i| MISCLICKS.get(i as usize))
        .unwrap_or(&['\0'; N_MISCLICKS])
        .iter()
        .copied()
        .filter(|c| *c != '\0')
}

/// The returned iterator variants of `ch` with diacritics, variants of other registers (except for
/// case), e.g. for a base Katakana character, yields its variants in Hiragana, with Dakuten &
/// Handakuten.
#[allow(clippy::too_many_lines, reason = "how else u gonna write this lol")]
pub fn variants(ch: char) -> impl Iterator<Item = char> + Clone {
    let chars: &[char] = match ch {
        // Latin
        'a' => &['â', 'ã', 'ä', 'à', 'á', 'ą', 'ā', 'Â', 'Ã', 'Ä', 'À', 'Á', 'Ą', 'Ā'],
        'c' => &['ć', 'č', 'ç', 'Ć', 'Č', 'Ç'],
        'd' => &['ď', 'đ', 'ð', 'Ď', 'Đ', 'Ð'],
        'e' => &['ê', 'ë', 'è', 'é', 'ę', 'ē', 'Ê', 'Ë', 'È', 'É', 'Ę', 'Ē'],
        'g' => &['ğ', 'ģ', 'Ğ', 'Ģ'],
        'h' => &['ĥ', 'Ĥ'],
        'i' => &['î', 'ï', 'ì', 'í', 'ī', 'į', 'ĩ', 'ı', 'İ', 'Î', 'Ï', 'Ì', 'Í', 'Ī', 'Į', 'Ĩ', 'I', 'İ'],
        'j' => &['ĵ', 'Ĵ'],
        'k' => &['ķ', 'Ķ'],
        'l' => &['ĺ', 'ļ', 'ľ', 'ł', 'Ĺ', 'Ļ', 'Ľ', 'Ł'],
        'n' => &['ñ', 'ń', 'ň', 'ņ', 'Ñ', 'Ń', 'Ň', 'Ņ'],
        'o' => &['ô', 'õ', 'ö', 'ò', 'ó', 'ø', 'ō', 'ő', 'Ô', 'Õ', 'Ö', 'Ò', 'Ó', 'Ø', 'Ō', 'Ő'],
        'r' => &['ř', 'ŕ', 'ŗ', 'Ř', 'Ŕ', 'Ŗ'],
        's' => &['ś', 'š', 'ş', 'ș', 'ß', 'Ś', 'Š', 'Ş', 'Ș', 'ẞ'],
        't' => &['ť', 'ţ', 'ț', 'Ť', 'Ţ', 'Ț'],
        'u' => &['û', 'ü', 'ù', 'ú', 'ū', 'ű', 'Û', 'Ü', 'Ù', 'Ú', 'Ū', 'Ű'],
        'w' => &['ŵ', 'Ŵ'],
        'y' => &['ŷ', 'ÿ', 'ý', 'Ŷ', 'Ÿ', 'Ý'],
        'z' => &['ž', 'ź', 'ż', 'Ž', 'Ź', 'Ż'],

        // Cyrillic
        'е' => &['ё', 'Ё'],
        'и' => &['й', 'Й'],
        'і' => &['ї', 'Ї'],
        'у' => &['ў', 'Ў'],
        'ь' => &['ъ', 'Ъ'],

        // Greek
        'α' => &['ά', 'Ά'],
        'ε' => &['έ', 'Έ'],
        'η' => &['ή', 'Ή'],
        'ι' => &['ί', 'ϊ', 'ΐ', 'Ί', 'Ϊ', 'ΐ'],
        'ο' => &['ό', 'Ό'],
        'υ' => &['ύ', 'ϋ', 'ΰ', 'Ύ', 'Ϋ', 'ΰ'],
        'ω' => &['ώ', 'Ώ'],

        // Hiragana
        'あ' => &['ア', 'ぁ', 'ァ'],
        'い' => &['イ', 'ぃ', 'ィ'],
        'う' => &['ウ', 'ぅ', 'ゥ'],
        'え' => &['エ', 'ぇ', 'ェ'],
        'お' => &['オ', 'ぉ', 'ォ'],
        'か' => &['カ', 'が', 'ガ', 'ゕ', 'ヵ'],
        'き' => &['キ', 'ぎ', 'ギ'],
        'く' => &['ク', 'ぐ', 'グ'],
        'け' => &['ケ', 'げ', 'ゲ', 'ゖ', 'ヶ'],
        'こ' => &['コ', 'ご', 'ゴ'],
        'さ' => &['サ', 'ざ', 'ザ'],
        'し' => &['シ', 'じ', 'ジ'],
        'す' => &['ス', 'ず', 'ズ'],
        'せ' => &['セ', 'ぜ', 'ゼ'],
        'そ' => &['ソ', 'ぞ', 'ゾ'],
        'た' => &['タ', 'だ', 'ダ'],
        'ち' => &['チ', 'ぢ', 'ヂ'],
        'つ' => &['ツ', 'づ', 'ヅ', 'っ', 'ッ'],
        'て' => &['テ', 'で', 'デ'],
        'と' => &['ト', 'ど', 'ド'],
        'な' => &['ナ'],
        'に' => &['ニ'],
        'ぬ' => &['ヌ'],
        'ね' => &['ネ'],
        'の' => &['ノ'],
        'は' => &['ハ', 'ば', 'バ', 'ぱ', 'パ'],
        'ひ' => &['ヒ', 'び', 'ビ', 'ぴ', 'ピ'],
        'ふ' => &['フ', 'ぶ', 'ブ', 'ぷ', 'プ'],
        'へ' => &['ヘ', 'べ', 'ベ', 'ぺ', 'ペ'],
        'ほ' => &['ホ', 'ぼ', 'ボ', 'ぽ', 'ポ'],
        'ま' => &['マ'],
        'み' => &['ミ'],
        'む' => &['ム'],
        'め' => &['メ'],
        'も' => &['モ'],
        'や' => &['ヤ', 'ゃ', 'ャ'],
        'ゆ' => &['ユ', 'ゅ', 'ュ'],
        'よ' => &['ヨ', 'ょ', 'ョ'],
        'ら' => &['ラ'],
        'り' => &['リ'],
        'る' => &['ル'],
        'れ' => &['レ'],
        'ろ' => &['ロ'],
        'わ' => &['ワ', 'ゎ', 'ヮ'],
        'を' => &['ヲ'],
        'ん' => &['ン'],

        // Katakana
        'ア' => &['あ', 'ぁ', 'ァ'],
        'イ' => &['い', 'ぃ', 'ィ'],
        'ウ' => &['う', 'ぅ', 'ゥ'],
        'エ' => &['え', 'ぇ', 'ェ'],
        'オ' => &['お', 'ぉ', 'ォ'],
        'カ' => &['か', 'が', 'ゕ', 'ヵ', 'ガ'],
        'キ' => &['き', 'ぎ', 'ギ'],
        'ク' => &['く', 'ぐ', 'グ'],
        'ケ' => &['け', 'げ', 'ゖ', 'ヶ', 'ゲ'],
        'コ' => &['こ', 'ご', 'ゴ'],
        'サ' => &['さ', 'ざ', 'ザ'],
        'シ' => &['し', 'じ', 'ジ'],
        'ス' => &['す', 'ず', 'ズ'],
        'セ' => &['せ', 'ぜ', 'ゼ'],
        'ソ' => &['そ', 'ぞ', 'ゾ'],
        'タ' => &['た', 'だ', 'ダ'],
        'チ' => &['ち', 'ぢ', 'ヂ'],
        'ツ' => &['つ', 'づ', 'ヅ', 'っ', 'ッ'],
        'テ' => &['て', 'で', 'デ'],
        'ト' => &['と', 'ど', 'ド'],
        'ナ' => &['な'],
        'ニ' => &['に'],
        'ヌ' => &['ぬ'],
        'ネ' => &['ね'],
        'ノ' => &['の'],
        'ハ' => &['は', 'ば', 'ぱ', 'バ', 'パ'],
        'ヒ' => &['ひ', 'び', 'ぴ', 'ビ', 'ピ'],
        'フ' => &['ふ', 'ぶ', 'ぷ', 'ブ', 'プ'],
        'ヘ' => &['へ', 'べ', 'ぺ', 'ベ', 'ペ'],
        'ホ' => &['ほ', 'ぼ', 'ぽ', 'ボ', 'ポ'],
        'マ' => &['ま'],
        'ミ' => &['み'],
        'ム' => &['む'],
        'メ' => &['め'],
        'モ' => &['も'],
        'ヤ' => &['や', 'ゃ', 'ャ'],
        'ユ' => &['ゆ', 'ゅ', 'ュ'],
        'ヨ' => &['よ', 'ょ', 'ョ'],
        'ラ' => &['ら'],
        'リ' => &['り'],
        'ル' => &['る'],
        'レ' => &['れ'],
        'ロ' => &['ろ'],
        'ワ' => &['わ', 'ゎ', 'ヮ'],
        'ヲ' => &['を'],
        'ン' => &['ん'],

        _ => &[],
    };
    chars.iter().copied()
}

/// Returns an iterator that combines all iterators over lookalikes defined in this module.
pub fn all(ch: char) -> impl Iterator<Item = char> + Clone {
    qwerty_misclicks(ch).chain(variants(ch))
}
