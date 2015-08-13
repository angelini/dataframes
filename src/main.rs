use std::fmt;

#[derive(Clone, Debug)]
enum Action<'a> {
    Empty,
    None,
    Name(&'a str),
    Select,
    Map,
    Filter,
    Group(u32),
    Join(&'a str),
}

type Step<'a> = Vec<Vec<Action<'a>>>;

#[derive(Clone, Debug)]
struct Query<'a> {
    steps: Step<'a>,
}

impl<'a> Query<'a> {
    fn width(&self) -> usize {
        match self.steps.last() {
            Some(actions) => actions.len(),
            None => 0
        }
    }

    fn optimize(&self) -> Query {
        self.clone()
    }
}

impl<'a> fmt::Display for Query<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for step in &self.steps {
            for action in step {
                let string = format!("{:?}", action);
                let col = format!("{:<11}", string);
                try!(write!(f, "{}", col))
            }
            try!(writeln!(f, ""))
        };
        Ok(())
    }
}

fn main() {
    let query = Query {
        steps: vec![
            vec![Action::Name("a"), Action::Name("b"), Action::Name("c")],
            vec![Action::Map,       Action::Map,       Action::Map],
            vec![Action::None,      Action::None,      Action::Filter],
            vec![Action::Join("d"), Action::None,      Action::None,      Action::Name("d"), Action::Name("e")],
            vec![Action::Group(0),  Action::None,      Action::None,      Action::None,      Action::None],
            vec![Action::Empty,     Action::Select,    Action::Empty,     Action::Select,    Action::Empty]
            ]
    };
    let optimized = query.optimize();
    println!("-> Query: \n{}", query);
    println!("-> Optimized: \n{}", optimized);
}
