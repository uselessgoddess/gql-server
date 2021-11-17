use std::borrow::Cow;
use std::cell::Cell;
use std::marker::PhantomData;
use std::sync::Arc;

use async_graphql::{
    InputObject,
    InputType, Object,
    OutputType,
    SimpleObject, Type,
};
use async_graphql::registry::Registry;
use async_std::sync::RwLock;
use doublets::doublets::ILinks;
use doublets::doublets::ILinksExtensions;
use doublets::num::LinkType;

pub struct Query<T: LinkType, Links: ILinks<T>> {
    pub links: Arc<RwLock<Links>>,
    _phantom: PhantomData<T>,
}

#[Object]
impl<T: LinkType + OutputType, Links: ILinks<T> + Sync + Send> Query<T, Links>  {
    #[graphql(skip)]
    pub fn new(links: Arc<RwLock<Links>>) -> Self {
        Self {
            links,
            _phantom: PhantomData
        }
    }

    #[graphql(name = "links")]
    async fn links(&self, /* todo: args */) -> Vec<Link<T>> {
        let mut links = self.links.read().await;

        let mut returning = Vec::with_capacity(links.count().as_());
        let constants = links.constants();
        links.each(|link| {
            returning.push(Link {
                id: link.index,
                from_id: link.source,
                to_id: link.target
            });
            constants.r#continue
        });
        returning
    }
}

pub struct Mutation<T: LinkType, Links: ILinks<T>> {
    pub links: Arc<RwLock<Links>>,
    _phantom: PhantomData<T>,
}

#[derive(InputObject)]
#[graphql(concrete(name = "InputLinkUsize", params(usize)))]
struct InputLink<T: LinkType + Type + InputType> {
    #[graphql(name = "from_id")] pub from_id: T,
    #[graphql(name = "to_id")] pub to_id: T,
}

#[derive(SimpleObject)]
struct Link<T: LinkType + Type + OutputType> {
    #[graphql(name = "id")] pub id: T,
    #[graphql(name = "from_id")] pub from_id: T,
    #[graphql(name = "to_id")] pub to_id: T,
}

#[Object]
impl<Links: ILinks<usize> + Sync + Send>  Mutation<usize, Links>  {
    #[graphql(skip)]
    pub fn new(links: Arc<RwLock<Links>>) -> Self {
        Self {
            links,
            _phantom: PhantomData
        }
    }

    #[graphql(name = "insert_links")]
    async fn insert_links(&self, objects: Vec<InputLink<usize>>) -> Vec<Link<usize>> {
        let mut links = self.links.write().await;

        let mut returning = Vec::with_capacity(objects.len());
        for link in objects {
            let new = links.get_or_create(link.from_id, link.to_id);
            returning.push(Link {
                id: new,
                from_id: link.from_id,
                to_id: link.to_id
            })
        }
        returning
    }
}
