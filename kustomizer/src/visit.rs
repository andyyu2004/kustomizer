use std::ops::ControlFlow;

pub trait VisitorMut {
    type Break;

    fn visit_value(&mut self, node: &mut serde_yaml::Value) -> ControlFlow<Self::Break> {
        self.walk_value(node)
    }

    fn visit_mapping(&mut self, map: &mut serde_yaml::Mapping) -> ControlFlow<Self::Break> {
        self.walk_mapping(map)
    }

    fn visit_sequence(&mut self, seq: &mut serde_yaml::Sequence) -> ControlFlow<Self::Break> {
        self.walk_sequence(seq)
    }

    fn walk_mapping(&mut self, map: &mut serde_yaml::Mapping) -> ControlFlow<Self::Break> {
        for value in map.values_mut() {
            // No mutable reference to the key
            self.visit_value(value)?;
        }

        ControlFlow::Continue(())
    }

    fn walk_sequence(&mut self, seq: &mut serde_yaml::Sequence) -> ControlFlow<Self::Break> {
        for item in seq.iter_mut() {
            self.visit_value(item)?;
        }

        ControlFlow::Continue(())
    }

    fn walk_value(&mut self, node: &mut serde_yaml::Value) -> ControlFlow<Self::Break> {
        match node {
            serde_yaml::Value::Null
            | serde_yaml::Value::Bool(_)
            | serde_yaml::Value::Number(_)
            | serde_yaml::Value::String(_)
            | serde_yaml::Value::Tagged(_) => ControlFlow::Continue(()),
            serde_yaml::Value::Sequence(seq) => self.visit_sequence(seq),
            serde_yaml::Value::Mapping(map) => self.visit_mapping(map),
        }
    }
}

pub trait VisitMut {
    fn visit_with<V: VisitorMut>(&mut self, visitor: &mut V) -> ControlFlow<V::Break>;
}

impl VisitMut for serde_yaml::Value {
    fn visit_with<V: VisitorMut>(&mut self, visitor: &mut V) -> ControlFlow<V::Break> {
        visitor.visit_value(self)
    }
}
