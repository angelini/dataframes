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

#[derive(Clone, Debug, PartialEq)]
pub struct Step<'a> {
    actions: Vec<Action<'a>>,
}

impl<'a> Step<'a> {
    fn new(actions: Vec<Action<'a>>) -> Step<'a> {
        Step { actions: actions }
    }

    fn is_filter(&self) -> bool {
        self.actions.contains(&Action::Filter)
    }

    fn widest_filter_index(&self) -> Option<usize> {
        for (i, action) in self.actions.iter().enumerate().rev() {
            println!("--> i {}", i);
            match action {
                &Action::Filter => { return Some(i) },
                _ => {}
            }
        };
        return None
    }

    fn is_group(&self) -> bool {
        for action in &self.actions {
            match action {
                &Action::Group(_) => { return true },
                _ => {},
            }
        };
        return false
    }
}

#[derive(Clone, Debug, PartialEq)]
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
    fn new(step_vec: Vec<Vec<Action<'a>>>) -> Query<'a> {
        let steps = step_vec.into_iter().map(|actions| Step::new(actions)).collect();
        Query { steps: steps }
    }

    fn width(&self) -> usize {
        match self.steps.last() {
            Some(&Step { ref actions }) => actions.len(),
            None => 0
        }
    }

    fn col(&self, index: usize) -> Col<'a> {
        let actions = self.steps.iter().map(|step| {
            match step.actions.get(index) {
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

        let mut filter_anchor = 0;
        for (i, step) in query.steps.clone().iter().enumerate() {
            if step.is_group() {
                filter_anchor = i
            }

            if step.is_filter() {
                for j in i..filter_anchor {
                    if query.steps[j].actions.len() < (&query.steps[i].widest_filter_index().unwrap() - 1) {
                        filter_anchor = j;
                        break
                    }
                };
                query.raise_step(i, filter_anchor)
            }
        };

        query
    }

    fn remove_col(&mut self, index: usize) {
        for step in &mut self.steps {
            if index < step.actions.len() {
                step.actions.remove(index);
            }
        }
    }

    fn raise_step(&mut self, index: usize, anchor: usize) {
        let rows_to_move_up = (anchor + 2..index + 1).rev();
        for i in rows_to_move_up {
            self.steps.swap(i, i - 1)
        }
    }
}

impl<'a> fmt::Display for Query<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for step in &self.steps {
            for action in &step.actions {
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

    #[test]
    fn optimize_will_move_filters_upwards() {
        let query = Query::new(vec![
            vec![Action::Name("a")],
            vec![Action::Map],
            vec![Action::Filter],
            ]);
        assert_eq!(query.optimize(), Query::new(vec![
            vec![Action::Name("a")],
            vec![Action::Filter],
            vec![Action::Map],
            ]))
    }

    #[test]
    fn step_can_find_the_widest_filter_action() {
        let step = Step::new(vec![
            Action::None, Action::Filter, Action::Filter, Action::None,
            ]);
        assert_eq!(step.widest_filter_index().unwrap(), 2)
    }
}
