#![allow(dead_code)]

use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum Action<'a> {
    Empty,
    None,
    Name(&'a str),
    Select,
    Map,
    Filter,
    Group(u32),
    Join(&'a str),
}

type Step<'a> = Vec<Action<'a>>;

#[derive(Debug, PartialEq)]
pub struct Col<'a> {
    actions: Vec<Action<'a>>,
}

impl<'a> Col<'a> {
    fn new(actions: Vec<Action<'a>>) -> Col<'a> {
        Col { actions: actions }
    }

    fn is_empty(&self) -> bool {
        let mut is_empty = false;
        let mut is_used = false;
        let mut seen_name = false;

        for action in &self.actions {
            match action {
                &Action::Empty => {
                    if seen_name && !is_used {
                        is_empty = true
                    }
                },
                &Action::Name(_) => seen_name = true,
                &Action::Filter => is_used = true,
                &Action::Join(_) => is_used = true,
                _ => {},
            }
        };
        is_empty
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Query<'a> {
    steps: Vec<Step<'a>>,
}

impl<'a> Query<'a> {
    fn new(steps: Vec<Step<'a>>) -> Query<'a> {
        Query { steps: steps }
    }

    fn width(&self) -> usize {
        match self.steps.last() {
            Some(actions) => actions.len(),
            None => 0
        }
    }

    fn col(&self, index: usize) -> Col<'a> {
        let actions = self.steps.iter().map(|step| {
            match step.get(index) {
                Some(action) => action.clone(),
                None => Action::Empty,
            }
        }).collect();
        Col::new(actions)
    }

    fn cols(&self) -> Vec<Col<'a>> {
        (0..self.width()).map(|i| {
            self.col(i)
        }).collect()
    }

    fn optimize(&self) -> Query {
        let mut query = self.clone();
        for (i, col) in query.cols().iter().enumerate() {
            if col.is_empty() {
                query.remove_col(i)
            }
        };
        query
    }

    fn remove_col(&mut self, index: usize) {
        for step in &mut self.steps {
            if index < step.len() {
                step.remove(index);
            }
        }
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
    let query = Query::new(vec![
        vec![Action::Name("a"), Action::Name("b"), Action::Name("c")],
        vec![Action::Map,       Action::Map,       Action::Map],
        vec![Action::None,      Action::None,      Action::Filter],
        vec![Action::Join("d"), Action::None,      Action::None,      Action::Name("d"), Action::Name("e")],
        vec![Action::Group(0),  Action::None,      Action::None,      Action::None,      Action::None],
        vec![Action::Empty,     Action::Select,    Action::Empty,     Action::Select,    Action::Empty],
        ]
    );
    let optimized = query.optimize();
    println!("-> Query: \n{}", query);
    println!("-> Optimized: \n{}", optimized);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_select_column_from_query() {
        let query = Query::new(vec![
            vec![Action::Name("a")],
            vec![Action::Join("d"), Action::Name("b")],
            ]
        );
        assert_eq!(query.col(0).actions, vec![Action::Name("a"), Action::Join("d")]);
        assert_eq!(query.col(1).actions, vec![Action::Empty, Action::Name("b")]);
    }

    #[test]
    fn can_select_all_columns_from_query() {
        let query = Query::new(vec![
            vec![Action::Name("a")],
            vec![Action::Join("d"), Action::Name("b")],
            ]
        );
        assert_eq!(query.cols(), vec![
            Col::new(vec![Action::Name("a"), Action::Join("d")]),
            Col::new(vec![Action::Empty, Action::Name("b")]),
        ])
    }

    #[test]
    fn can_detect_empty_col() {
        assert!(
            !Col::new(vec![Action::Name("a")]).is_empty());
        assert!(
            Col::new(vec![Action::Empty, Action::Name("a"), Action::Empty]).is_empty());
        assert!(
            !Col::new(vec![Action::Empty, Action::Name("a"), Action::Join("d"), Action::Empty]).is_empty())
    }

    #[test]
    fn optimize_will_remove_an_empty_col() {
        let query = Query::new(vec![
            vec![Action::Name("a")],
            vec![Action::Join("d"), Action::Name("b"), Action::Name("c")],
            vec![Action::Select,    Action::Select,    Action::Empty],
            ]);
        assert_eq!(query.optimize(), Query::new(vec![
            vec![Action::Name("a")],
            vec![Action::Join("d"), Action::Name("b")],
            vec![Action::Select,    Action::Select],
            ]))
    }
}
