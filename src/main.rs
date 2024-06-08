use serde::{Deserialize, Serialize};
use std::error::Error;
use surrealdb::engine::local::{Db, Mem};
use surrealdb::sql::Thing;
use surrealdb::Surreal;

pub trait IdTraits {
    fn get_tbl(&self) -> String;
    fn get_id(&self) -> String;
    fn get_tbl_id(&self) -> String;
}

// Note the id returned by get_id and get_tbl_id maybe begin and end
// with `⟨` and `⟩`. This happen when the id field is a decimal
// integer number passed as a string. Note those are NOT '<' and '>'
// characters!
impl IdTraits for Thing {
    fn get_tbl(&self) -> String {
        self.tb.to_string()
    }

    fn get_id(&self) -> String {
        self.id.to_string()
    }

    fn get_tbl_id(&self) -> String {
        self.to_string()
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct BuildingWithThing {
    id: Thing,
    address: String,
}

async fn test_select_thing_with_id_traits(db: &Surreal<Db>, address: &str, tbl: &str, id: &str) -> Result<(), Box<dyn Error>> {
    let select_results_with_id_traits: Vec<BuildingWithThing> = db.select("building_tbl").await?;
    dbg!(&select_results_with_id_traits);
    assert!(select_results_with_id_traits.len() >= 1);
    assert_eq!(&select_results_with_id_traits[0].id.get_tbl(), tbl);
    assert_eq!(&select_results_with_id_traits[0].id.get_id(), id);
    assert_eq!(&select_results_with_id_traits[0].id.get_tbl_id(), &(tbl.to_owned() + ":" + id));
    assert_eq!(&select_results_with_id_traits[0].address, address);

    //println!("id: {:?}", select_results_with_id_traits[0].id);
    //println!("id.id: {:?}", select_results_with_id_traits[0].id.id);
    //let id_get_tbl_id = select_results_with_id_traits[0].id.get_tbl_id();
    //println!("id_get_tbl_id: {}", id_get_tbl_id);
    //let id_get_id = select_results_with_id_traits[0].id.get_id();
    //println!("id_get_id: {}", id_get_id);
    //let id_get_tbl = select_results_with_id_traits[0].id.get_tbl();
    //println!("id_get_tbl: {}", id_get_tbl);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create a new SurrealDB instance
    let db = Surreal::new::<Mem>(()).await?;
    dbg!(&db);

    db.use_ns("test").use_db("test").await?;

    // Add a record with a `rid` and `address` fields
    let table = "building_tbl";
    let address = "123 Main St";
    let mut created_response = db
        // OK same as building_tbl:rand()
        //.query(r#"CREATE building_tbl SET address = $addr;"#)

        // OK
        //.query(r#"CREATE building_tbl:rand() SET address = $addr;"#)

        // OK
        //.query(r#"CREATE building_tbl:uuid() SET address = $addr;"#)

        // OK
        //.query(r#"CREATE building_tbl:ulid() SET address = $addr;"#)
        //.query(r#"CREATE building_tbl:ulid() SET address = $addr;"#)

        // OK
        //.query(r#"CREATE building_tbl:123 SET address = $addr;"#)

        // OK, two string array
        //.query(r#"CREATE building_tbl:['london', "a1234"] SET address = $addr;"#)

        // OK, two string array
        //.query(r#"CREATE building_tbl:['london', "1234"] SET address = $addr;"#)

        // OK, two element array with string and number
        //.query(r#"CREATE building_tbl:['london', 1234] SET address = $addr;"#)

        // OK, two string array but the id will be rendered as a String with ⟨1234⟩ as contents
        .query(r#"CREATE building_tbl SET id = "123", address = $addr;"#)

        // BUG? InvalidQuery(RenderedError) (The above is OK, this should work too)
        //.query(r#"CREATE building_tbl:"1234" SET address = $addr;"#)

        // Valid Err? InvalidQuery(RenderedError) 
        //.query(r#"LET now = time::now(); CREATE building_tbl:$now SET address = $addr;"#)

        // BUG in deserilize? OK, but fails to in `result.as_ref().unwrap()` to BuildingWithThing
        //.query(r#"LET $now = time::now(); CREATE building_tbl:['london', $now] SET address = $addr;"#)

        // BUG in deserialize? OK, but fails to in `result.as_ref().unwrap()` to BuildingWithThing
        //.query(r#"LET $now = time::now(); CREATE building_tbl:{ city: 'london', time: $now} SET address = $addr;"#)

        // BUG? OK, two element array with string and None
        //.query(r#"CREATE building_tbl:['london', a1234] SET address = $addr;"#)

        .bind(("addr", address))
        //.bind(("rid", r#"['london', $now"]"#))
        .await?;
    dbg!(&created_response);
    let result: Option<BuildingWithThing> = created_response.take(0)?;
    dbg!(&result);

    // Get the generated `rid`;
    let bwt = result.as_ref().unwrap();
    let rid = bwt.id.get_id();
    dbg!(&rid);

    test_select_thing_with_id_traits(&db, address, table, &rid).await?;

    Ok(())
}
