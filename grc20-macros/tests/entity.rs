use grc20_core::{
    mapping::{triple, Attributes, FromAttributes, IntoAttributes, PropFilter, Query, QueryStream},
    neo4rs, system_ids,
};

mod test_ids {
    pub const PERSON_TYPE: &str = "PERSON";
    pub const NAME_ATTR: &str = "name";
    pub const AGE_ATTR: &str = "age";
    pub const NICKNAME_ATTRIBUTE: &str = "nickname";
}

#[test]
fn test_entity_macro_with_string_literals() {
    #[grc20_core::entity]
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

#[test]
fn test_entity_macro_with_paths() {
    #[grc20_core::entity]
    #[grc20(schema_type = test_ids::PERSON_TYPE)]
    struct Person {
        #[grc20(attribute = test_ids::NAME_ATTR)]
        name: String,
        #[grc20(attribute = test_ids::NICKNAME_ATTRIBUTE)]
        nickname: Option<String>,
        #[grc20(attribute = test_ids::AGE_ATTR)]
        age: String,
    }

    // Test IntoAttributes
    let person = Person {
        name: "Alice".to_string(),
        nickname: Some("Ali".to_string()),
        age: "30".to_string(),
    };

    let attrs = person.into_attributes().unwrap();
    assert_eq!(attrs.get::<String>(test_ids::NAME_ATTR).unwrap(), "Alice");
    assert_eq!(
        attrs.get::<String>(test_ids::NICKNAME_ATTRIBUTE).unwrap(),
        "Ali"
    );
    assert_eq!(attrs.get::<u64>(test_ids::AGE_ATTR).unwrap(), 30);

    // Test FromAttributes
    let attrs = Attributes::default()
        .attribute((test_ids::NAME_ATTR, "Bob".to_string()))
        .attribute((test_ids::AGE_ATTR, 25u64));

    let person = Person::from_attributes(attrs).unwrap();
    assert_eq!(person.name, "Bob");
    assert_eq!(person.nickname, None);
    assert_eq!(person.age, "25");
}

use futures::{pin_mut, StreamExt};
use testcontainers::{
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
    GenericImage, ImageExt,
};

const BOLT_PORT: u16 = 7687;
const HTTP_PORT: u16 = 7474;

#[derive(Clone)]
#[grc20_core::entity]
#[grc20(schema_type = test_ids::PERSON_TYPE)]
struct Person {
    #[grc20(attribute = test_ids::NAME_ATTR)]
    name: String,
    #[grc20(attribute = test_ids::NICKNAME_ATTRIBUTE)]
    nickname: Option<String>,
    #[grc20(attribute = test_ids::AGE_ATTR)]
    age: u64,
}

#[tokio::test]
async fn test_find_one() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let container = GenericImage::new("neo4j", "2025.01.0-community")
        .with_wait_for(WaitFor::Duration {
            length: std::time::Duration::from_secs(5),
        })
        .with_exposed_port(BOLT_PORT.tcp())
        .with_exposed_port(HTTP_PORT.tcp())
        .with_env_var("NEO4J_AUTH", "none")
        .start()
        .await
        .expect("Failed to start Neo 4J container");

    let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
    let host = container.get_host().await.unwrap().to_string();

    let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "user", "password")
        .await
        .unwrap();

    let person = Person {
        name: "Alice".into(),
        nickname: Some("Ali".into()),
        age: 30,
    };

    let entity =
        grc20_core::mapping::Entity::new("abc", person.clone()).with_type(test_ids::PERSON_TYPE);

    entity
        .clone()
        .insert(
            &neo4j,
            &grc20_core::block::BlockMetadata::default(),
            "ROOT",
            "0",
        )
        .send()
        .await
        .expect("Failed to insert entity");

    let found_entity = find_one(&neo4j, "abc", "ROOT")
        .send()
        .await
        .expect("Failed to find entity")
        .expect("Entity not found");

    assert_eq!(found_entity.id(), "abc");
    assert_eq!(found_entity.attributes.name, person.name);
    assert_eq!(found_entity.attributes.nickname, person.nickname);
    assert_eq!(found_entity.attributes.age, person.age);
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_find_many() {
    // Setup a local Neo 4J container for testing. NOTE: docker service must be running.
    let container = GenericImage::new("neo4j", "2025.01.0-community")
        .with_wait_for(WaitFor::Duration {
            length: std::time::Duration::from_secs(5),
        })
        .with_exposed_port(BOLT_PORT.tcp())
        .with_exposed_port(HTTP_PORT.tcp())
        .with_env_var("NEO4J_AUTH", "none")
        .start()
        .await
        .expect("Failed to start Neo 4J container");

    let port = container.get_host_port_ipv4(BOLT_PORT).await.unwrap();
    let host = container.get_host().await.unwrap().to_string();

    let neo4j = neo4rs::Graph::new(format!("neo4j://{host}:{port}"), "user", "password")
        .await
        .unwrap();

    // Create person type entity
    triple::insert_one(
        &neo4j,
        &grc20_core::block::BlockMetadata::default(),
        "ROOT",
        "0",
        triple::Triple::new("PERSON", system_ids::NAME_ATTRIBUTE, "Person"),
    )
    .send()
    .await
    .expect("Failed to insert triple");

    // Create TYPES relation type
    triple::insert_one(
        &neo4j,
        &grc20_core::block::BlockMetadata::default(),
        "ROOT",
        "0",
        triple::Triple::new(
            system_ids::TYPES_ATTRIBUTE,
            system_ids::NAME_ATTRIBUTE,
            "Types",
        ),
    )
    .send()
    .await
    .expect("Failed to insert triple");

    let person = Person {
        name: "Alice".into(),
        nickname: Some("Ali".into()),
        age: 30,
    };

    let entity =
        grc20_core::mapping::Entity::new("abc", person.clone()).with_type(test_ids::PERSON_TYPE);

    entity
        .clone()
        .insert(
            &neo4j,
            &grc20_core::block::BlockMetadata::default(),
            "ROOT",
            "0",
        )
        .send()
        .await
        .expect("Failed to insert entity");

    let stream = find_many(&neo4j, "ROOT")
        .name(PropFilter::default().value("Alice"))
        .limit(1)
        .send()
        .await
        .expect("Failed to find entity");

    pin_mut!(stream);

    let found_entity = stream
        .next()
        .await
        .expect("Failed to get next entity")
        .expect("Entity not found");

    assert_eq!(found_entity.id(), "abc");
    assert_eq!(found_entity.attributes.name, person.name);
    assert_eq!(found_entity.attributes.nickname, person.nickname);
    assert_eq!(found_entity.attributes.age, person.age);
}
