use std::{collections::HashMap, str::{from_utf8, Utf8Error}};

use emojis::Emoji;
use lazy_static::lazy_static;

lazy_static! {
    static ref SMILEY_TO_EMOJI_MAP: HashMap<&'static str, &'static str> = HashMap::from([
        (":)", "🙂"),
        (":D", "😁"),
        (";)", "😉"),
        (":-O", "😮"),
        (":P", "😋"),
        ("(H)", "😎"),
        (":@", "😡"),
        (":$", "😳"),
        (":S", "😵‍💫"),
        (":(", "🙁"),
        (":'(", "😭"),
        (":|", "😐️"),
        ("(6)", "😈"),
        ("(A)", "😇"),
        ("(L)", "❤️"),
        ("(U)", "💔"),
        ("(M)", "💬"),
        ("(@)", "🐱"),
        ("(&)", "🐶"),
        ("(S)", "🌜️"),
        ("(*)", "⭐️"),
        ("(~)", "🎞️"),
        ("(8)", "🎵"),
        ("(E)", "📧"),
        ("(F)", "🌹"),
        ("(W)", "🥀"),
        ("(O)", "🕒"),
        ("(K)", "💋"),
        ("(G)", "🎁"),
        ("(^)", "🎂"),
        ("(P)", "📷"),
        ("(I)", "💡"),
        ("(C)", "☕"),
        ("(T)", "📞"),
        ("({)", "🧍‍♂️"),
        ("(})", "🧍🏾‍♀️"),
        ("(B)", "🍺"),
        ("(D)", "🍸"),
        ("(Z)", "🧍‍♂️"),
        ("(X)", "🧍‍♀️"),
        ("(Y)", "👍"),
        ("(N)", "👎"),
        (":[", "🦇"),
        ("(nnh)", "🐐"),
        ("(#)", "☀️"),
        ("(R)", "🌈"),
        (":-#", "🤐"),
        ("8o|", "😬"),
        ("8-|", "🤓"),
        ("^o)", "🤨"),
        (":-*", "🤐"),
        ("+o(", "🤮"),
        ("(sn)", "🐌"),
        ("(tu)", "🐢"),
        ("(pl)","🍽️"),
        ("(||)", "🥣"),
        ("(pi)", "🍕"),
        ("(so)", "⚽"),
        ("(au)", "🚗"),
        ("(ap)", "✈️"),
        ("(um)", "☂️"),
        ("(ip)", "🏝️"),
        ("(co)", "🖥️"),
        ("(mp)", "📱"),
        ("(brb)", "👋"),
        ("(st)", "🌧️"),
        ("(h5)", "🖐️"),
        ("(mo)", "🪙"),
        ("(bah)", "🐑"),
        (":^)", "🤔"),
        ("*-)", "🤔"),
        ("(li)", "🌩️"),
        ("<:o)", "🥳"),
        ("8-)", "🙄"),
        ("|-)", "😴"),
        ("('.')", "🐰")
    ]);

    static ref EMOJI_TO_SMILEY_MAP: HashMap<&'static str, &'static str> = HashMap::from([
        ("🙂", ":)"),
        ("😁", ":D"),
        ("😉", ";)"),
        ("😮", ":-O"),
        ("😋", ":P"),
        ("😎", "(H)"),
        ("😡", ":@"),
        ("😵‍💫", ":S"),
        ("😳", ":$"),
        ("🙁", ":(")
    ]);


}

pub fn smiley_to_emoji(msg: &String) -> String {
    let mut msg = msg.to_owned();
    for (key, value) in SMILEY_TO_EMOJI_MAP.iter() {
        msg = msg.replace(key, value);
    }
    return msg;
}

pub fn emoji_to_smiley(msg: &String) -> String {
    let mut msg = msg.to_owned();
    for (key, value) in EMOJI_TO_SMILEY_MAP.iter() {
        msg = msg.replace(key, value);
    }
    return msg;
}

#[cfg(test)]
mod tests {
    use log::info;

    use crate::utils::emoji::{smiley_to_emoji, emoji_to_smiley};



    #[test]
    fn test_smileys_to_emoji() {
        let smileyface = smiley_to_emoji(&String::from("hi:);)"));
        println!("emojis: {}", &smileyface);
       // let test = smileyface.as_str()).unwrap();
    }

    
    #[test]
    fn test_emojis_to_smiley() {
        let smileyface = emoji_to_smiley(&String::from("hi🙂😉"));
        println!("smiley: {}", &smileyface);
       // let test = smileyface.as_str()).unwrap();
    }

}
