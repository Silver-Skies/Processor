use utility::Bracket;

pub enum Control {
    Bracket(Bracket),
    End,
    EndOfFile
}
// 
// impl Representable for Control {
//     fn representation() -> Cow<'a, str> {
//         todo!()
//     }
// }

pub enum Symbol {
    Control(Control)
}

impl Symbol {
    
}

pub struct Token {

}