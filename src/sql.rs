use log::debug;
use rusqlite::Connection;

use std::collections::HashMap;

pub use crate::Irept;

pub struct SqlWriter {
    connection: Connection,
    irep_container: HashMap<Irept, usize>,
    string_ref_container: HashMap<String, usize>,
}

impl SqlWriter {
    pub fn write_to_file(symbols: Vec<Irept>, functions: Vec<(String, Irept)>, output: &str) {
        let mut writer = SqlWriter {
            connection: Connection::open(output).unwrap(),
            irep_container: HashMap::new(),
            string_ref_container: HashMap::new(),
        };

        writer
            .connection
            .execute(
                "create table String (id integer primary key, value text)",
                (),
            )
            .unwrap();
        writer
            .connection
            .execute(
                "create table Irep (id integer primary key,  value integer);",
                (),
            )
            .unwrap();
        writer.connection.execute("create table IrepSub (id integer primary key autoincrement, parent integer, child integer);", ()).unwrap();
        writer.connection.execute("create table IrepNamedSub (id integer primary key autoincrement, parent integer, child integer, name integer);", ()).unwrap();
        writer.connection.execute("create table IrepCommentSub (id integer primary key autoincrement, parent integer, child integer, name integer);", ()).unwrap();
        writer
            .connection
            .execute(
                "create table Symbol (id  integer primary key , irep integer);",
                (),
            )
            .unwrap();
        writer
            .connection
            .execute(
                "create table Function (id integer primary key, irep integer, name text);",
                (),
            )
            .unwrap();

        let mut counter = 0;
        for irep in symbols {
            debug!("Writing symbol: {}", irep.id);
            let id = writer.write_reference(&irep);
            // The ordering might be important!
            writer
                .connection
                .execute(
                    "INSERT INTO Symbol (id, irep) VALUES (?1, ?2)",
                    (counter, id),
                )
                .unwrap();
            counter = counter + 1;
        }

        counter = 0;
        for (name, irep) in functions {
            let id = writer.write_reference(&irep);
            writer
                .connection
                .execute(
                    "INSERT INTO Function (id, irep, name) VALUES (?1, ?2, ?3)",
                    (counter, id, name),
                )
                .unwrap();
            counter = counter + 1;
        }
    }

    fn write_string_reference(&mut self, value: &str) -> usize {
        if self.string_ref_container.contains_key(value) {
            let id = self.string_ref_container[value];
            return id;
        }
        let id = self.string_ref_container.len();
        self.string_ref_container.insert(String::from(value), id);
        self.connection
            .execute(
                "INSERT INTO String (id, value) VALUES (?1, ?2)",
                (id, value),
            )
            .unwrap();
        return id;
    }
    fn write_reference(&mut self, value: &Irept) -> usize {
        if self.irep_container.contains_key(value) {
            let id = self.irep_container[value];
            return id;
        }
        let id = self.irep_container.len();
        self.irep_container.insert(value.clone(), id);

        let string_ref = self.write_string_reference(&value.id);
        self.connection
            .execute(
                "INSERT INTO Irep (id, value) VALUES (?1, ?2)",
                (id, string_ref),
            )
            .unwrap();

        for irep in &value.subt {
            let sub_id = self.write_reference(irep);
            self.connection
                .execute(
                    "INSERT INTO IrepSub (parent, child) VALUES (?1, ?2)",
                    (id, sub_id),
                )
                .unwrap();
        }

        for (name, irep) in &value.named_subt {
            let sub_name = self.write_string_reference(name);
            let sub_id = self.write_reference(irep);
            self.connection
                .execute(
                    "INSERT INTO IrepNamedSub (parent, child, name) VALUES (?1, ?2, ?3)",
                    (id, sub_id, sub_name),
                )
                .unwrap();
        }

        for (name, irep) in &value.comments {
            let sub_name = self.write_string_reference(name);
            let sub_id = self.write_reference(irep);
            self.connection
                .execute(
                    "INSERT INTO IrepCommentSub (parent, child, name) VALUES (?1, ?2, ?3)",
                    (id, sub_id, sub_name),
                )
                .unwrap();
        }

        return id;
    }
}

pub struct SqlReader {
    connection: Connection,
}

impl SqlReader {
    pub fn open(path: &str) -> Self {
        SqlReader {
            connection: Connection::open(path).unwrap(),
        }
    }

    pub fn get_symbols(&self) -> Vec<Irept> {
        let mut stmt = self.connection.prepare("SELECT irep FROM Symbol").unwrap();
        let mut rows = stmt.query([]).unwrap();

        let mut symbols = Vec::new();

        while let Some(row) = rows.next().unwrap() {
            let id: usize = row.get(0).unwrap();
            symbols.push(self.read_irep(id));
        }

        symbols
    }

    pub fn get_functions(&self) -> Vec<(String, Irept)> {
        let mut stmt = self
            .connection
            .prepare("SELECT irep,name FROM Function")
            .unwrap();
        let mut rows = stmt.query([]).unwrap();

        let mut functions = Vec::new();

        while let Some(row) = rows.next().unwrap() {
            let id: usize = row.get(0).unwrap();
            let name: String = row.get(1).unwrap();
            functions.push((name, self.read_irep(id)));
        }

        functions
    }

    fn read_string(&self, string_id: usize) -> String {
        let id: String = self
            .connection
            .query_row(
                "SELECT value FROM String WHERE id=(?1)",
                [string_id],
                |row| row.get(0),
            )
            .unwrap();
        id
    }

    fn read_irep(&self, table_id: usize) -> Irept {
        // Get id
        let string_id: usize = self
            .connection
            .query_row("SELECT value FROM Irep WHERE id=(?1)", [table_id], |row| {
                row.get(0)
            })
            .unwrap();

        let id: String = self.read_string(string_id);

        // Get sub
        let mut subt: Vec<Irept> = Vec::new();
        let mut subt_stmt = self
            .connection
            .prepare("SELECT child FROM IrepSub WHERE IrepSub.parent=(?1)")
            .unwrap();

        let mut subt_rows = subt_stmt.query([table_id]).unwrap();
        while let Some(row) = subt_rows.next().unwrap() {
            let sub_id: usize = row.get(0).unwrap();
            subt.push(self.read_irep(sub_id));
        }

        // Get named sub
        let mut named_subt: HashMap<String, Irept> = HashMap::new();
        let mut named_stmt = self
            .connection
            .prepare("SELECT name,child FROM IrepNamedSub WHERE IrepNamedSub.parent=(?1)")
            .unwrap();
        let mut named_rows = named_stmt.query([table_id]).unwrap();
        while let Some(row) = named_rows.next().unwrap() {
            let name_id: usize = row.get(0).unwrap();
            let sub_id: usize = row.get(1).unwrap();
            named_subt.insert(self.read_string(name_id), self.read_irep(sub_id));
        }

        // Get comments
        let mut comments: HashMap<String, Irept> = HashMap::new();
        let mut comments_stmt = self
            .connection
            .prepare("SELECT name,child FROM IrepCommentSub WHERE IrepCommentSub.parent=(?1)")
            .unwrap();
        let mut comments_rows = comments_stmt.query([table_id]).unwrap();
        while let Some(row) = comments_rows.next().unwrap() {
            let name_id: usize = row.get(0).unwrap();
            let sub_id: usize = row.get(1).unwrap();
            comments.insert(self.read_string(name_id), self.read_irep(sub_id));
        }

        // Result
        Irept {
            id,
            subt,
            named_subt,
            comments,
        }
    }
}
