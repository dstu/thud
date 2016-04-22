use ::gtk;
use ::glib;
use ::mcts;
use ::search_graph;

pub struct Store {
    store: gtk::TreeStore,
    columns: Vec<Column>,
}

#[derive(Clone, Copy, Debug)]
pub enum Column {
    Id,
    Statistics,
    Action,
}

impl Column {
    pub fn glib_type(self) -> glib::types::Type {
        glib::types::Type::String
    }

    pub fn label(&self) -> &str {
        match *self {
            Column::Id => "Id",
            Column::Statistics => "Statistics",
            Column::Action => "Action",
        }
    }

    pub fn node_value<'a>(self, n: &mcts::ThudNode<'a>) -> glib::Value {
        unsafe {
            let mut v = glib::Value::new();
            v.init(self.glib_type());
            match self {
                Column::Id =>
                    v.set_string(format!("node:{}", n.get_id()).as_str()),
                Column::Statistics =>
                    v.set_string(""),
                Column::Action =>
                    v.set_string(""),
            }
            v
        }
    }
        
    pub fn edge_value<'a>(self, e: &mcts::ThudEdge<'a>) -> glib::Value {
        unsafe {
            let mut v = glib::Value::new();
            v.init(self.glib_type());
            match self {
                Column::Id =>
                    v.set_string(format!("edge:{}", e.get_id()).as_str()),
                Column::Statistics =>
                    v.set_string(format!("{:?}", e.get_data().statistics).as_str()),
                Column::Action =>
                    v.set_string(format!("{:?}", e.get_data().action).as_str()),
            }
            v
        }
    }

    pub fn new_view_column(self, col_number: i32) -> gtk::TreeViewColumn {
        let c = gtk::TreeViewColumn::new().unwrap();
        let cell = gtk::CellRendererText::new().unwrap();
        c.set_title(self.label());
        c.pack_start(&cell, true);
        c.add_attribute(&cell, "text", col_number);
        c
    }
}

// TODO: cycle detection.
impl Store {
    pub fn new(columns: &[Column]) -> Self {
        let template: Vec<glib::types::Type> = columns.iter().map(|c| c.glib_type()).collect();
        Store {
            store: gtk::TreeStore::new(template.as_slice()).unwrap(),
            columns: columns.iter().map(|x| *x).collect(),
        }
    }

    pub fn model(&self) -> gtk::TreeModel {
        self.store.get_model().unwrap()
    }

    pub fn columns(&self) -> &[Column] {
        self.columns.as_slice()
    }

    pub fn update<'a>(&mut self, root: mcts::ThudNode<'a>) {
        self.store.clear();

        let mut nodes = vec![(root, self.store.append(None))];
        while !nodes.is_empty() {
            let (n, parent) = nodes.pop().unwrap();
            self.set_node_columns(&n, &parent);
            // let children = n.get_child_list();
            // for c in 0..children.len() {
            //     let e = children.get_edge(c);
            //     let e_i = self.store.append(Some(&parent));
            //     self.set_edge_columns(&e, &e_i);
            //     nodes.push((parent.get_target(), self.store.append(Some(&e_i))));
            // }
        }
    }

    fn set_node_columns<'a>(&self, n: &mcts::ThudNode<'a>, i: &gtk::TreeIter) {
        for (col_number, col) in self.columns.iter().enumerate() {
            self.store.set_value(i, col_number as i32, &col.node_value(n));
        }
    }

    fn set_edge_columns<'a>(&self, e: &mcts::ThudEdge<'a>, i: &gtk::TreeIter) {
        for (col_number, col) in self.columns.iter().enumerate() {
            self.store.set_value(i, col_number as i32, &col.edge_value(e));
        }
    }
}
