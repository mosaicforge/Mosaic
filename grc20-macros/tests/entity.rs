use sdk::mapping::{Attributes, FromAttributes, IntoAttributes};

mod test_ids {
    pub const PERSON_TYPE: &str = "PERSON";
    pub const NAME_ATTR: &str = "name";
    pub const AGE_ATTR: &str = "age";
    pub const NICKNAME_ATTRIBUTE: &str = "nickname";
}

#[test]
fn test_entity_macro_with_string_literals() {
    #[grc20_macros::entity]
    #[grc20(schema_type = "PERSON")]
    struct Person {
        #[grc20(attribute = "name")]
        name: String,
        #[grc20(attribute = "nickname")]
        nickname: Option<String>,
        #[grc20(attribute = "age")]
        age: u64,
    }

    // Test IntoAttributes
    let person = Person {
        name: "Alice".to_string(),
        nickname: Some("Ali".to_string()),
        age: 30,
    };

    let attrs = person.into_attributes().unwrap();
    assert_eq!(attrs.get::<String>("name").unwrap(), "Alice");
    assert_eq!(attrs.get::<String>("nickname").unwrap(), "Ali");
    assert_eq!(attrs.get::<u64>("age").unwrap(), 30);

    // Test FromAttributes
    let attrs = Attributes::default()
        .attribute(("name", "Bob".to_string()))
        .attribute(("age", 25u64));

    let person = Person::from_attributes(attrs).unwrap();
    assert_eq!(person.name, "Bob");
    assert_eq!(person.nickname, None);
    assert_eq!(person.age, 25);
}

// #[test]
// fn test_entity_macro_builder() {
//     #[grc20_macros::entity]
//     #[grc20(schema_type = "PERSON")]
//     struct Person {
//         #[grc20(attribute = "name")]
//         name: String,
//         #[grc20(attribute = "nickname")]
//         nickname: Option<String>,
//         #[grc20(attribute = "age")]
//         age: u64,
//     }

//     // Test builder pattern
//     let person = Person::new("person-1")
//         .name("Alice")
//         .nickname("Ali".to_string())
//         .age(30)
//         .build();

//     assert_eq!(person.id, "person-1");
//     assert_eq!(person.attributes.name, "Alice");
//     assert_eq!(person.attributes.nickname, Some("Ali".to_string()));
//     assert_eq!(person.attributes.age, 30);
//     assert_eq!(person.types, vec!["PERSON".to_string()]);

//     // Test builder with default values
//     let person = Person::new("person-2").name("Bob").age(25).build();
    
//     assert_eq!(person.id, "person-2");
//     assert_eq!(person.attributes.name, "Bob");
//     assert_eq!(person.attributes.nickname, None);
//     assert_eq!(person.attributes.age, 25);
//     assert_eq!(person.types, vec!["PERSON".to_string()]);
// }

#[test]
fn test_entity_macro_with_paths() {
    #[grc20_macros::entity]
    #[grc20(schema_type = test_ids::PERSON_TYPE)]
    struct Person {
        #[grc20(attribute = test_ids::NAME_ATTR)]
        name: String,
        #[grc20(attribute = test_ids::NICKNAME_ATTRIBUTE)]
        nickname: Option<String>,
        #[grc20(attribute = test_ids::AGE_ATTR)]
        age: u64,
    }

    // Test IntoAttributes
    let person = Person {
        name: "Alice".to_string(),
        nickname: Some("Ali".to_string()),
        age: 30,
    };

    let attrs = person.into_attributes().unwrap();
    assert_eq!(attrs.get::<String>(test_ids::NAME_ATTR).unwrap(), "Alice");
    assert_eq!(attrs.get::<String>(test_ids::NICKNAME_ATTRIBUTE).unwrap(), "Ali");
    assert_eq!(attrs.get::<u64>(test_ids::AGE_ATTR).unwrap(), 30);

    // Test FromAttributes
    let attrs = Attributes::default()
        .attribute((test_ids::NAME_ATTR, "Bob".to_string()))
        .attribute((test_ids::AGE_ATTR, 25u64));

    let person = Person::from_attributes(attrs).unwrap();
    assert_eq!(person.name, "Bob");
    assert_eq!(person.nickname, None);
    assert_eq!(person.age, 25);
}
