use uuid::Uuid;
use serde_json;


table! {
    baskets (id) {
        id -> Uuid,
        contents -> Jsonb,
    }
}



#[derive(Serialize, Deserialize, Debug)]
struct BasketContentsV1(u32);
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct BasketContentsV2(u64);

version_json_type!(
    #[derive(Debug, Default)]
    basket_contents BasketContents {
        V1 => BasketContentsV1 {
            BasketContentsV2(basket_contents.0 as u64 + 10)
        },
        V2 => BasketContentsV2 {}
    }
);

#[derive(Queryable, Insertable, Default, Debug)]
#[table_name="baskets"]
pub struct Basket {
    pub id: Uuid,
    pub contents: BasketContents
}
