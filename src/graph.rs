use std::collections::HashMap;

pub struct Graph {
    pub map: HashMap<String, Vec<String>>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph {
            map: HashMap::new(),
        }
    }

    pub fn seen(&self, file_name: String) -> bool {
        self.map.contains_key(&file_name)
    }

    pub fn push_local_dep(&mut self, file_name: String, dep: String) {
        self._push(file_name, dep);
    }

    pub fn push_library_dep(&mut self, file_name: String, dep: String) {
        self._push(file_name, dep);
    }

    fn _push(&mut self, file_name: String, dep: String) {
        if self.map.contains_key(&file_name) {
            let m = &mut self.map;
            //let m = self.map.get(&file_name).unwrap();
            if let Some(v) = m.get_mut(&file_name) {
                v.push(dep);
            }
        } else {
            let v = vec![dep];
            self.map.insert(file_name.clone(), v);
        }
    }
}