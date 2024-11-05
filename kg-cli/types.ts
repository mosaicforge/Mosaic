[2m2024-10-29T21:59:10.986626Z[0m [33m WARN[0m [2mneo4rs::connection[0m[2m:[0m This driver does not yet implement client-side routing. It is possible that operations against a cluster (such as Aura) will fail.    
[2m2024-10-29T21:59:10.986693Z[0m [32m INFO[0m [2mneo4rs::pool[0m[2m:[0m creating connection pool with max size 16    
[2m2024-10-29T21:59:11.011008Z[0m [32m INFO[0m [2mneo4rs::pool[0m[2m:[0m creating new connection...    
[2m2024-10-29T21:59:11.463886Z[0m [32m INFO[0m [1mCompiler::print[0m[2m:[0m [2mswc_timer[0m[2m:[0m Done in 8.790637ms [3mkind[0m[2m=[0m"perf"
class WebUrl {
}
class BlockType {
    image: string;
    entityPage() {
        return neo4j.query("MATCH ({id: $id}) -[r:ENTITY_PAGE]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Space {
    cover: Image;
    broaderSpaces() {
        return neo4j.query("MATCH ({id: $id}) -[r:BROADER_SPACES]-> (n) RETURN n", {
            id: this.id
        });
    }
    foreignTypes() {
        return neo4j.query("MATCH ({id: $id}) -[r:FOREIGN_TYPES]-> (n) RETURN n", {
            id: this.id
        });
    }
    blocks() {
        return neo4j.query("MATCH ({id: $id}) -[r:BLOCKS]-> (n) RETURN n", {
            id: this.id
        });
    }
    blocks() {
        return neo4j.query("MATCH ({id: $id}) -[r:BLOCKS]-> (n) RETURN n", {
            id: this.id
        });
    }
    subspaces() {
        return neo4j.query("MATCH ({id: $id}) -[r:SUBSPACES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Relation {
}
class TableBlock {
    placeholderText: string;
    rowType() {
        return neo4j.query("MATCH ({id: $id}) -[r:ROW_TYPE]-> (n) RETURN n", {
            id: this.id
        });
    }
    entityPage() {
        return neo4j.query("MATCH ({id: $id}) -[r:ENTITY_PAGE]-> (n) RETURN n", {
            id: this.id
        });
    }
    view() {
        return neo4j.query("MATCH ({id: $id}) -[r:VIEW]-> (n) RETURN n", {
            id: this.id
        });
    }
    view() {
        return neo4j.query("MATCH ({id: $id}) -[r:VIEW]-> (n) RETURN n", {
            id: this.id
        });
    }
    shownColumns() {
        return neo4j.query("MATCH ({id: $id}) -[r:SHOWN_COLUMNS]-> (n) RETURN n", {
            id: this.id
        });
    }
    placeholderImage() {
        return neo4j.query("MATCH ({id: $id}) -[r:PLACEHOLDER_IMAGE]-> (n) RETURN n", {
            id: this.id
        });
    }
    hiddenColumns() {
        return neo4j.query("MATCH ({id: $id}) -[r:HIDDEN_COLUMNS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Relation {
    index: string;
    fromEntity() {
        return neo4j.query("MATCH ({id: $id}) -[r:FROM_ENTITY]-> (n) RETURN n", {
            id: this.id
        });
    }
    relationTypes() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATION_TYPES]-> (n) RETURN n", {
            id: this.id
        });
    }
    toEntity() {
        return neo4j.query("MATCH ({id: $id}) -[r:TO_ENTITY]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Text {
}
class Person {
    cover: Image;
    avatar: Image;
}
class TextBlock {
    markdownContent: string;
    entityPage() {
        return neo4j.query("MATCH ({id: $id}) -[r:ENTITY_PAGE]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Image {
    imageUrl: Web URL;
    fileType: string;
    height: string;
    width: string;
    author() {
        return neo4j.query("MATCH ({id: $id}) -[r:AUTHOR]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Date {
}
class Attribute {
    valueType() {
        return neo4j.query("MATCH ({id: $id}) -[r:VALUE_TYPE]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class RelationValueTypes {
}
class Type {
    template() {
        return neo4j.query("MATCH ({id: $id}) -[r:TEMPLATE]-> (n) RETURN n", {
            id: this.id
        });
    }
    template() {
        return neo4j.query("MATCH ({id: $id}) -[r:TEMPLATE]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class View {
}
class Region {
}
class PodcastEpisode {
    podcast() {
        return neo4j.query("MATCH ({id: $id}) -[r:PODCAST]-> (n) RETURN n", {
            id: this.id
        });
    }
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    publishDate() {
        return neo4j.query("MATCH ({id: $id}) -[r:PUBLISH_DATE]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    guests() {
        return neo4j.query("MATCH ({id: $id}) -[r:GUESTS]-> (n) RETURN n", {
            id: this.id
        });
    }
    hosts() {
        return neo4j.query("MATCH ({id: $id}) -[r:HOSTS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Place {
    streetAddress: string;
    types() {
        return neo4j.query("MATCH ({id: $id}) -[r:TYPES]-> (n) RETURN n", {
            id: this.id
        });
    }
    categories() {
        return neo4j.query("MATCH ({id: $id}) -[r:CATEGORIES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Tutorial {
    cover: Image;
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsAbout() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_ABOUT]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    quotesFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:QUOTES_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    bias() {
        return neo4j.query("MATCH ({id: $id}) -[r:BIAS]-> (n) RETURN n", {
            id: this.id
        });
    }
    transcripts() {
        return neo4j.query("MATCH ({id: $id}) -[r:TRANSCRIPTS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class GovernmentEntity {
    cover: Image;
    avatar: Image;
    streetAddress: string;
    email: string;
    types() {
        return neo4j.query("MATCH ({id: $id}) -[r:TYPES]-> (n) RETURN n", {
            id: this.id
        });
    }
    goals() {
        return neo4j.query("MATCH ({id: $id}) -[r:GOALS]-> (n) RETURN n", {
            id: this.id
        });
    }
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    regions() {
        return neo4j.query("MATCH ({id: $id}) -[r:REGIONS]-> (n) RETURN n", {
            id: this.id
        });
    }
    demographicsServed() {
        return neo4j.query("MATCH ({id: $id}) -[r:DEMOGRAPHICS_SERVED]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    providedServices() {
        return neo4j.query("MATCH ({id: $id}) -[r:PROVIDED_SERVICES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Quote {
    sourceUrl: Web URL;
    source() {
        return neo4j.query("MATCH ({id: $id}) -[r:SOURCE]-> (n) RETURN n", {
            id: this.id
        });
    }
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    author() {
        return neo4j.query("MATCH ({id: $id}) -[r:AUTHOR]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsAbout() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_ABOUT]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Policy {
    types() {
        return neo4j.query("MATCH ({id: $id}) -[r:TYPES]-> (n) RETURN n", {
            id: this.id
        });
    }
    goals() {
        return neo4j.query("MATCH ({id: $id}) -[r:GOALS]-> (n) RETURN n", {
            id: this.id
        });
    }
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Nonprofit {
    cover: Image;
    avatar: Image;
    streetAddress: string;
    nonprofitId: string;
    email: string;
    types() {
        return neo4j.query("MATCH ({id: $id}) -[r:TYPES]-> (n) RETURN n", {
            id: this.id
        });
    }
    goals() {
        return neo4j.query("MATCH ({id: $id}) -[r:GOALS]-> (n) RETURN n", {
            id: this.id
        });
    }
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    nonprofitCategories() {
        return neo4j.query("MATCH ({id: $id}) -[r:NONPROFIT_CATEGORIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    regions() {
        return neo4j.query("MATCH ({id: $id}) -[r:REGIONS]-> (n) RETURN n", {
            id: this.id
        });
    }
    demographicsServed() {
        return neo4j.query("MATCH ({id: $id}) -[r:DEMOGRAPHICS_SERVED]-> (n) RETURN n", {
            id: this.id
        });
    }
    demographicsServed() {
        return neo4j.query("MATCH ({id: $id}) -[r:DEMOGRAPHICS_SERVED]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    providedServices() {
        return neo4j.query("MATCH ({id: $id}) -[r:PROVIDED_SERVICES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class PressRelease {
    avatar: Image;
    claimsAbout() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_ABOUT]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    quotesFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:QUOTES_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedNewsEvents() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_NEWS_EVENTS]-> (n) RETURN n", {
            id: this.id
        });
    }
    publishDate() {
        return neo4j.query("MATCH ({id: $id}) -[r:PUBLISH_DATE]-> (n) RETURN n", {
            id: this.id
        });
    }
    publisher() {
        return neo4j.query("MATCH ({id: $id}) -[r:PUBLISHER]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedNewsTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_NEWS_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedNewsStories() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_NEWS_STORIES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class NewsTopic {
    cover: Image;
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    broaderTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:BROADER_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    editors() {
        return neo4j.query("MATCH ({id: $id}) -[r:EDITORS]-> (n) RETURN n", {
            id: this.id
        });
    }
    editors() {
        return neo4j.query("MATCH ({id: $id}) -[r:EDITORS]-> (n) RETURN n", {
            id: this.id
        });
    }
    subNewsTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:SUB_NEWS_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedNewsTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_NEWS_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedNewsStories() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_NEWS_STORIES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Politician {
    cover: Image;
    avatar: Image;
    votingPositions() {
        return neo4j.query("MATCH ({id: $id}) -[r:VOTING_POSITIONS]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    positions() {
        return neo4j.query("MATCH ({id: $id}) -[r:POSITIONS]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    cosponsoredLegislation() {
        return neo4j.query("MATCH ({id: $id}) -[r:COSPONSORED_LEGISLATION]-> (n) RETURN n", {
            id: this.id
        });
    }
    sponsoredLegislation() {
        return neo4j.query("MATCH ({id: $id}) -[r:SPONSORED_LEGISLATION]-> (n) RETURN n", {
            id: this.id
        });
    }
    bioguideId() {
        return neo4j.query("MATCH ({id: $id}) -[r:BIOGUIDE_ID]-> (n) RETURN n", {
            id: this.id
        });
    }
    politicalParty() {
        return neo4j.query("MATCH ({id: $id}) -[r:POLITICAL_PARTY]-> (n) RETURN n", {
            id: this.id
        });
    }
    enactedLegislation() {
        return neo4j.query("MATCH ({id: $id}) -[r:ENACTED_LEGISLATION]-> (n) RETURN n", {
            id: this.id
        });
    }
    pastRoles() {
        return neo4j.query("MATCH ({id: $id}) -[r:PAST_ROLES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Dao {
    avatar: Image;
    mission() {
        return neo4j.query("MATCH ({id: $id}) -[r:MISSION]-> (n) RETURN n", {
            id: this.id
        });
    }
    vision() {
        return neo4j.query("MATCH ({id: $id}) -[r:VISION]-> (n) RETURN n", {
            id: this.id
        });
    }
    values() {
        return neo4j.query("MATCH ({id: $id}) -[r:VALUES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Publisher {
    cover: Image;
    avatar: Image;
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    owners() {
        return neo4j.query("MATCH ({id: $id}) -[r:OWNERS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedArticles() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ARTICLES]-> (n) RETURN n", {
            id: this.id
        });
    }
    bias() {
        return neo4j.query("MATCH ({id: $id}) -[r:BIAS]-> (n) RETURN n", {
            id: this.id
        });
    }
    sections() {
        return neo4j.query("MATCH ({id: $id}) -[r:SECTIONS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Concept {
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    bias() {
        return neo4j.query("MATCH ({id: $id}) -[r:BIAS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Document {
    author() {
        return neo4j.query("MATCH ({id: $id}) -[r:AUTHOR]-> (n) RETURN n", {
            id: this.id
        });
    }
    quotesFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:QUOTES_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    publishDate() {
        return neo4j.query("MATCH ({id: $id}) -[r:PUBLISH_DATE]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedDocuments() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_DOCUMENTS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Conference {
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class HomelessAssistance {
    cover: Image;
    avatar: Image;
}
class Topic {
    cover: Image;
    types() {
        return neo4j.query("MATCH ({id: $id}) -[r:TYPES]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    broaderTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:BROADER_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    subtopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:SUBTOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    speakers() {
        return neo4j.query("MATCH ({id: $id}) -[r:SPEAKERS]-> (n) RETURN n", {
            id: this.id
        });
    }
    definitions() {
        return neo4j.query("MATCH ({id: $id}) -[r:DEFINITIONS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Airport {
    streetAddress: string;
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Question {
    sources() {
        return neo4j.query("MATCH ({id: $id}) -[r:SOURCES]-> (n) RETURN n", {
            id: this.id
        });
    }
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    answers() {
        return neo4j.query("MATCH ({id: $id}) -[r:ANSWERS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class HomelessPrevention {
}
class LowIncome {
    cover: Image;
    avatar: Image;
}
class NewsStory {
    cover: Image;
    avatar: Image;
    date: Date;
    coverCredits: string;
    coverCredits: string;
    types() {
        return neo4j.query("MATCH ({id: $id}) -[r:TYPES]-> (n) RETURN n", {
            id: this.id
        });
    }
    sources() {
        return neo4j.query("MATCH ({id: $id}) -[r:SOURCES]-> (n) RETURN n", {
            id: this.id
        });
    }
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsAbout() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_ABOUT]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    furtherReading() {
        return neo4j.query("MATCH ({id: $id}) -[r:FURTHER_READING]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedNewsEvents() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_NEWS_EVENTS]-> (n) RETURN n", {
            id: this.id
        });
    }
    broaderTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:BROADER_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    summary() {
        return neo4j.query("MATCH ({id: $id}) -[r:SUMMARY]-> (n) RETURN n", {
            id: this.id
        });
    }
    broaderNewsTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:BROADER_NEWS_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relevantQuestions() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELEVANT_QUESTIONS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedNewsTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_NEWS_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedNewsTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_NEWS_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedNewsStories() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_NEWS_STORIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    primaryTopic() {
        return neo4j.query("MATCH ({id: $id}) -[r:PRIMARY_TOPIC]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Claim {
    sources() {
        return neo4j.query("MATCH ({id: $id}) -[r:SOURCES]-> (n) RETURN n", {
            id: this.id
        });
    }
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    quotesThatSupportTheClaim() {
        return neo4j.query("MATCH ({id: $id}) -[r:QUOTES_THAT_SUPPORT_THE_CLAIM]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    subclaims() {
        return neo4j.query("MATCH ({id: $id}) -[r:SUBCLAIMS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relevantQuestions() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELEVANT_QUESTIONS]-> (n) RETURN n", {
            id: this.id
        });
    }
    opposingArguments() {
        return neo4j.query("MATCH ({id: $id}) -[r:OPPOSING_ARGUMENTS]-> (n) RETURN n", {
            id: this.id
        });
    }
    quotesThatOpposeTheClaim() {
        return neo4j.query("MATCH ({id: $id}) -[r:QUOTES_THAT_OPPOSE_THE_CLAIM]-> (n) RETURN n", {
            id: this.id
        });
    }
    supportingArguments() {
        return neo4j.query("MATCH ({id: $id}) -[r:SUPPORTING_ARGUMENTS]-> (n) RETURN n", {
            id: this.id
        });
    }
    broaderClaims() {
        return neo4j.query("MATCH ({id: $id}) -[r:BROADER_CLAIMS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Tag {
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    broaderTags() {
        return neo4j.query("MATCH ({id: $id}) -[r:BROADER_TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    subtags() {
        return neo4j.query("MATCH ({id: $id}) -[r:SUBTAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Solution {
    problems() {
        return neo4j.query("MATCH ({id: $id}) -[r:PROBLEMS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Restaurant {
    cover: Image;
    cover: Image;
    avatar: Image;
    avatar: Image;
    streetAddress: string;
    cuisine: string;
    cuisine: string;
    instagram: Web URL;
}
class Podcast {
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    authors() {
        return neo4j.query("MATCH ({id: $id}) -[r:AUTHORS]-> (n) RETURN n", {
            id: this.id
        });
    }
    podcastEpisodes() {
        return neo4j.query("MATCH ({id: $id}) -[r:PODCAST_EPISODES]-> (n) RETURN n", {
            id: this.id
        });
    }
    hosts() {
        return neo4j.query("MATCH ({id: $id}) -[r:HOSTS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Article {
    cover: Image;
    avatar: Image;
    webArchiveUrl: Web URL;
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    author() {
        return neo4j.query("MATCH ({id: $id}) -[r:AUTHOR]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsAbout() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_ABOUT]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    quotesFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:QUOTES_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedNewsEvents() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_NEWS_EVENTS]-> (n) RETURN n", {
            id: this.id
        });
    }
    publishDate() {
        return neo4j.query("MATCH ({id: $id}) -[r:PUBLISH_DATE]-> (n) RETURN n", {
            id: this.id
        });
    }
    publisher() {
        return neo4j.query("MATCH ({id: $id}) -[r:PUBLISHER]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    authors() {
        return neo4j.query("MATCH ({id: $id}) -[r:AUTHORS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedNewsTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_NEWS_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedArticles() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ARTICLES]-> (n) RETURN n", {
            id: this.id
        });
    }
    bias() {
        return neo4j.query("MATCH ({id: $id}) -[r:BIAS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedNewsStories() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_NEWS_STORIES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Book {
    cover: Image;
    avatar: Image;
    originalTitle: string;
    copyright: string;
    abstract: string;
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    publishDate() {
        return neo4j.query("MATCH ({id: $id}) -[r:PUBLISH_DATE]-> (n) RETURN n", {
            id: this.id
        });
    }
    publisher() {
        return neo4j.query("MATCH ({id: $id}) -[r:PUBLISHER]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    authors() {
        return neo4j.query("MATCH ({id: $id}) -[r:AUTHORS]-> (n) RETURN n", {
            id: this.id
        });
    }
    genre() {
        return neo4j.query("MATCH ({id: $id}) -[r:GENRE]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Legislation {
    regions() {
        return neo4j.query("MATCH ({id: $id}) -[r:REGIONS]-> (n) RETURN n", {
            id: this.id
        });
    }
    votingResults() {
        return neo4j.query("MATCH ({id: $id}) -[r:VOTING_RESULTS]-> (n) RETURN n", {
            id: this.id
        });
    }
    regionalLevel() {
        return neo4j.query("MATCH ({id: $id}) -[r:REGIONAL_LEVEL]-> (n) RETURN n", {
            id: this.id
        });
    }
    cosponsors() {
        return neo4j.query("MATCH ({id: $id}) -[r:COSPONSORS]-> (n) RETURN n", {
            id: this.id
        });
    }
    status() {
        return neo4j.query("MATCH ({id: $id}) -[r:STATUS]-> (n) RETURN n", {
            id: this.id
        });
    }
    politicalInstitution() {
        return neo4j.query("MATCH ({id: $id}) -[r:POLITICAL_INSTITUTION]-> (n) RETURN n", {
            id: this.id
        });
    }
    sponsor() {
        return neo4j.query("MATCH ({id: $id}) -[r:SPONSOR]-> (n) RETURN n", {
            id: this.id
        });
    }
    votingIssues() {
        return neo4j.query("MATCH ({id: $id}) -[r:VOTING_ISSUES]-> (n) RETURN n", {
            id: this.id
        });
    }
    votingIssuePosition() {
        return neo4j.query("MATCH ({id: $id}) -[r:VOTING_ISSUE_POSITION]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Position {
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    publisher() {
        return neo4j.query("MATCH ({id: $id}) -[r:PUBLISHER]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    newsEvents() {
        return neo4j.query("MATCH ({id: $id}) -[r:NEWS_EVENTS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Service {
    subservices() {
        return neo4j.query("MATCH ({id: $id}) -[r:SUBSERVICES]-> (n) RETURN n", {
            id: this.id
        });
    }
    broaderServices() {
        return neo4j.query("MATCH ({id: $id}) -[r:BROADER_SERVICES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Church {
    avatar: Image;
    types() {
        return neo4j.query("MATCH ({id: $id}) -[r:TYPES]-> (n) RETURN n", {
            id: this.id
        });
    }
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    nonprofitCategories() {
        return neo4j.query("MATCH ({id: $id}) -[r:NONPROFIT_CATEGORIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    providedServices() {
        return neo4j.query("MATCH ({id: $id}) -[r:PROVIDED_SERVICES]-> (n) RETURN n", {
            id: this.id
        });
    }
    providedServices() {
        return neo4j.query("MATCH ({id: $id}) -[r:PROVIDED_SERVICES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Workstream {
    features() {
        return neo4j.query("MATCH ({id: $id}) -[r:FEATURES]-> (n) RETURN n", {
            id: this.id
        });
    }
    broaderWorkstreams() {
        return neo4j.query("MATCH ({id: $id}) -[r:BROADER_WORKSTREAMS]-> (n) RETURN n", {
            id: this.id
        });
    }
    subworkstreams() {
        return neo4j.query("MATCH ({id: $id}) -[r:SUBWORKSTREAMS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class OnlinePlatform {
    cover: Image;
    avatar: Image;
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Study {
    abstract: string;
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    opposingArguments() {
        return neo4j.query("MATCH ({id: $id}) -[r:OPPOSING_ARGUMENTS]-> (n) RETURN n", {
            id: this.id
        });
    }
    authors() {
        return neo4j.query("MATCH ({id: $id}) -[r:AUTHORS]-> (n) RETURN n", {
            id: this.id
        });
    }
    journal() {
        return neo4j.query("MATCH ({id: $id}) -[r:JOURNAL]-> (n) RETURN n", {
            id: this.id
        });
    }
    supportingArguments() {
        return neo4j.query("MATCH ({id: $id}) -[r:SUPPORTING_ARGUMENTS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Person {
    cover: Image;
    avatar: Image;
    wallets() {
        return neo4j.query("MATCH ({id: $id}) -[r:WALLETS]-> (n) RETURN n", {
            id: this.id
        });
    }
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    eventBadges() {
        return neo4j.query("MATCH ({id: $id}) -[r:EVENT_BADGES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Page {
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    pageType() {
        return neo4j.query("MATCH ({id: $id}) -[r:PAGE_TYPE]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    subpages() {
        return neo4j.query("MATCH ({id: $id}) -[r:SUBPAGES]-> (n) RETURN n", {
            id: this.id
        });
    }
    broaderPages() {
        return neo4j.query("MATCH ({id: $id}) -[r:BROADER_PAGES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class TrainingProvider {
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    coursesOffered() {
        return neo4j.query("MATCH ({id: $id}) -[r:COURSES_OFFERED]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class HomelessStreetPrograms {
    cover: Image;
    avatar: Image;
}
class Task {
    claimedBy() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMED_BY]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    inProgressSubtasks() {
        return neo4j.query("MATCH ({id: $id}) -[r:IN_PROGRESS_SUBTASKS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relevantPages() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELEVANT_PAGES]-> (n) RETURN n", {
            id: this.id
        });
    }
    completedSubtasks() {
        return neo4j.query("MATCH ({id: $id}) -[r:COMPLETED_SUBTASKS]-> (n) RETURN n", {
            id: this.id
        });
    }
    status() {
        return neo4j.query("MATCH ({id: $id}) -[r:STATUS]-> (n) RETURN n", {
            id: this.id
        });
    }
    reviewedBy() {
        return neo4j.query("MATCH ({id: $id}) -[r:REVIEWED_BY]-> (n) RETURN n", {
            id: this.id
        });
    }
    priority() {
        return neo4j.query("MATCH ({id: $id}) -[r:PRIORITY]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class IndexedSpace {
}
class Problem {
    solutions() {
        return neo4j.query("MATCH ({id: $id}) -[r:SOLUTIONS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Post {
    avatar: Image;
    avatar: Image;
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsAbout() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_ABOUT]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    publishDate() {
        return neo4j.query("MATCH ({id: $id}) -[r:PUBLISH_DATE]-> (n) RETURN n", {
            id: this.id
        });
    }
    publishDate() {
        return neo4j.query("MATCH ({id: $id}) -[r:PUBLISH_DATE]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    authors() {
        return neo4j.query("MATCH ({id: $id}) -[r:AUTHORS]-> (n) RETURN n", {
            id: this.id
        });
    }
    onlinePublicationPlatforms() {
        return neo4j.query("MATCH ({id: $id}) -[r:ONLINE_PUBLICATION_PLATFORMS]-> (n) RETURN n", {
            id: this.id
        });
    }
    subPost() {
        return neo4j.query("MATCH ({id: $id}) -[r:SUB_POST]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class TouristOffice {
}
class Metric {
    metricType() {
        return neo4j.query("MATCH ({id: $id}) -[r:METRIC_TYPE]-> (n) RETURN n", {
            id: this.id
        });
    }
    unit() {
        return neo4j.query("MATCH ({id: $id}) -[r:UNIT]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class 25c1b007e6a144c6aed1da2e0dd39a2c {
}
class Category {
    avatar: Image;
}
class NonprofitService {
    cover: Image;
    avatar: Image;
}
class Transcript {
    cover: Image;
    avatar: Image;
    sourceUrl: Web URL;
    source() {
        return neo4j.query("MATCH ({id: $id}) -[r:SOURCE]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsAbout() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_ABOUT]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    quotesFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:QUOTES_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class EventBadge {
}
class AbuseAndIntervention {
    cover: Image;
    avatar: Image;
}
class Course {
    topics() {
        return neo4j.query("MATCH ({id: $id}) -[r:TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    providedBy() {
        return neo4j.query("MATCH ({id: $id}) -[r:PROVIDED_BY]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class OpinionPoll {
    avatar: Image;
    claimsAbout() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_ABOUT]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    publishDate() {
        return neo4j.query("MATCH ({id: $id}) -[r:PUBLISH_DATE]-> (n) RETURN n", {
            id: this.id
        });
    }
    publisher() {
        return neo4j.query("MATCH ({id: $id}) -[r:PUBLISHER]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relevantQuestions() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELEVANT_QUESTIONS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class FinanceSummary {
    otherExpenses: string;
    otherRevenueSources: string;
    salariesBenefits: string;
    constructionCosts: string;
    programServicesFees: string;
    nonFinancialAssets: string;
    totalRevenue: string;
    contributions: string;
    grants: string;
}
class Abbbe7f19c4d4daca63ed7363ca9b9a4 {
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    subpages() {
        return neo4j.query("MATCH ({id: $id}) -[r:SUBPAGES]-> (n) RETURN n", {
            id: this.id
        });
    }
    broaderPages() {
        return neo4j.query("MATCH ({id: $id}) -[r:BROADER_PAGES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class News {
    cover: Image;
    avatar: Image;
}
class Activity {
    cover: Image;
    roles() {
        return neo4j.query("MATCH ({id: $id}) -[r:ROLES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class NewsEvent {
    date: Date;
    relatedNewsEvents() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_NEWS_EVENTS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedNewsTopics() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_NEWS_TOPICS]-> (n) RETURN n", {
            id: this.id
        });
    }
    relatedNewsStories() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_NEWS_STORIES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Mission {
}
class Controversy {
    politicians() {
        return neo4j.query("MATCH ({id: $id}) -[r:POLITICIANS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class HomelessReEntry {
    cover: Image;
    avatar: Image;
}
class Journal {
    avatar: Image;
    relatedEntities() {
        return neo4j.query("MATCH ({id: $id}) -[r:RELATED_ENTITIES]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class SocialServiceCategory {
    types() {
        return neo4j.query("MATCH ({id: $id}) -[r:TYPES]-> (n) RETURN n", {
            id: this.id
        });
    }
    subservices() {
        return neo4j.query("MATCH ({id: $id}) -[r:SUBSERVICES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class PricingType {
}
class Definition {
    publisher() {
        return neo4j.query("MATCH ({id: $id}) -[r:PUBLISHER]-> (n) RETURN n", {
            id: this.id
        });
    }
    tags() {
        return neo4j.query("MATCH ({id: $id}) -[r:TAGS]-> (n) RETURN n", {
            id: this.id
        });
    }
    definitionOf() {
        return neo4j.query("MATCH ({id: $id}) -[r:DEFINITION_OF]-> (n) RETURN n", {
            id: this.id
        });
    }
    authors() {
        return neo4j.query("MATCH ({id: $id}) -[r:AUTHORS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Vision {
}
class WorkingGroup {
}
class PoliticalInstitution {
    streetAddress: string;
    regions() {
        return neo4j.query("MATCH ({id: $id}) -[r:REGIONS]-> (n) RETURN n", {
            id: this.id
        });
    }
    regionalLevel() {
        return neo4j.query("MATCH ({id: $id}) -[r:REGIONAL_LEVEL]-> (n) RETURN n", {
            id: this.id
        });
    }
    governmentBranch() {
        return neo4j.query("MATCH ({id: $id}) -[r:GOVERNMENT_BRANCH]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Location {
}
class HomelessWorkers {
    cover: Image;
    avatar: Image;
}
class Stage {
}
class YesNo {
}
class Tab {
}
class Transportation {
    cover: Image;
    avatar: Image;
}
class Certification {
}
class UserStory {
}
class Debate {
    date: Date;
    claimsAbout() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_ABOUT]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    hosts() {
        return neo4j.query("MATCH ({id: $id}) -[r:HOSTS]-> (n) RETURN n", {
            id: this.id
        });
    }
    debaters() {
        return neo4j.query("MATCH ({id: $id}) -[r:DEBATERS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Conflict {
    participants() {
        return neo4j.query("MATCH ({id: $id}) -[r:PARTICIPANTS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class AdvocacyAndOutreach {
    cover: Image;
    avatar: Image;
}
class PoliticalEvent {
}
class FinanceOverview {
}
class Hotel {
}
class Value {
}
class Party {
    streetAddress: string;
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    claimsFrom() {
        return neo4j.query("MATCH ({id: $id}) -[r:CLAIMS_FROM]-> (n) RETURN n", {
            id: this.id
        });
    }
    positions() {
        return neo4j.query("MATCH ({id: $id}) -[r:POSITIONS]-> (n) RETURN n", {
            id: this.id
        });
    }
    positions() {
        return neo4j.query("MATCH ({id: $id}) -[r:POSITIONS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Organization {
    cover: Image;
    avatar: Image;
    mission() {
        return neo4j.query("MATCH ({id: $id}) -[r:MISSION]-> (n) RETURN n", {
            id: this.id
        });
    }
    vision() {
        return neo4j.query("MATCH ({id: $id}) -[r:VISION]-> (n) RETURN n", {
            id: this.id
        });
    }
    values() {
        return neo4j.query("MATCH ({id: $id}) -[r:VALUES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Feature {
    targetedQuarter() {
        return neo4j.query("MATCH ({id: $id}) -[r:TARGETED_QUARTER]-> (n) RETURN n", {
            id: this.id
        });
    }
    owners() {
        return neo4j.query("MATCH ({id: $id}) -[r:OWNERS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class PoliticalCampaign {
}
class AttendanceType {
}
class Objective {
    goals() {
        return neo4j.query("MATCH ({id: $id}) -[r:GOALS]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Flow {
}
class Initiative {
}
class Status {
}
class Product {
    price: string;
}
class 6d01d30a6fb24c9bbf6cc6d5d5ee6559 {
    subpages() {
        return neo4j.query("MATCH ({id: $id}) -[r:SUBPAGES]-> (n) RETURN n", {
            id: this.id
        });
    }
    broaderPages() {
        return neo4j.query("MATCH ({id: $id}) -[r:BROADER_PAGES]-> (n) RETURN n", {
            id: this.id
        });
    }
}
class Bug {
}
class TargetedQuarter {
}
class Election {
}

