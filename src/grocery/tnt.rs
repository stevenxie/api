use super::{Prices, Product, ProductIterator, Sailor};
use crate::common::*;

use json::Value as JsonValue;
use json_dotpath::DotPaths;

use std::fmt::{Display, Formatter, Result as FmtResult};

use bigdecimal::Zero;
use request::Client;
use tokio_compat::FutureExt;

/// A `TntSailor` finds sales at T&T Supermarket.
#[derive(Builder)]
pub struct TntSailor {
    #[builder(default = "build_client()")]
    client: Client,

    #[builder(default = "25")]
    page_size: u32,
}

fn build_client() -> Client {
    Client::builder().cookie_store(true).build().unwrap()
}

impl TntSailor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> TntSailorBuilder {
        TntSailorBuilder::default()
    }
}

impl Default for TntSailor {
    fn default() -> Self {
        Self::builder().build().unwrap()
    }
}

const TNT_API_URL: &str = "https://www.tntsupermarket.com/rest/V1";

#[async_trait]
impl Sailor for TntSailor {
    async fn get_sale_products(
        &self,
        postcode: String,
    ) -> Result<ProductIterator> {
        // Set location.
        // TODO: Split this into a separate helper function.
        let url =
            format!("{}/tntzone/location/getpreferedstorecode", TNT_API_URL);
        let postcode: String = postcode.chars().take(3).collect();
        let response = self
            .client
            .get(&url)
            .query(&[("postcode", &postcode)])
            .send()
            .compat()
            .await
            .context("failed to send request")?;
        let status = response.status();
        if !status.is_success() {
            warn!("failed to set preferred store: {}", &status)
        }

        // Get weekly specials.
        let url = format!("{}/xmapi/app-weekly-special", TNT_API_URL);
        let page = "1";
        let page_size = self.page_size.to_string();
        let response = self
            .client
            .get(&url)
            .query(&[("page", page), ("pageSize", &page_size)])
            .send()
            .compat()
            .await
            .context("failed to send request")?;
        let status = response.status();
        if !status.is_success() {
            bail!("bad response: {}", &status);
        }
        let value: JsonValue =
            response.json().await.context("failed to parse response")?;

        let products = value
            .dot_get::<Vec<TntProduct>>("data.category.items")
            .context("failed to parse response items")?
            .context("missing response items")?;
        let products: Vec<TntProduct> = products
            .into_iter()
            .filter(|product| {
                if (product.is_available == 0) {
                    info!("found an unavailable product: {}", product);
                    return false;
                }
                if (product.is_saleable == 0) {
                    info!("found an unsellable product: {}", product);
                    return false;
                }
                if (product.prices.final_price.amount.is_zero()) {
                    info!("found an unpriced product: {}", product);
                    return false;
                }
                true
            })
            .collect();
        let products = products.into_iter().map(Product::from);
        Ok(Box::new(products))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TntProduct {
    id: String,
    sku: String,
    name: String,
    prices: TntPrices,
    weight_uom: String,
    is_available: u8,
    is_saleable: u8,
}

impl Display for TntProduct {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_fmt(format_args!("{} (SKU: {})", &self.name, &self.sku))
    }
}

impl From<TntProduct> for Product {
    fn from(
        TntProduct {
            id,
            name,
            prices,
            weight_uom,
            sku,
            ..
        }: TntProduct,
    ) -> Self {
        Product {
            name,
            units: Some(weight_uom).filter(|string| !string.is_empty()),
            prices: prices.into(),
            vendor: "T&T".to_owned(),
            vendor_id: id,
            vendor_sku: sku,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TntPrices {
    old_price: TntPrice,
    final_price: TntPrice,
}

impl From<TntPrices> for Prices {
    fn from(
        TntPrices {
            old_price,
            final_price,
        }: TntPrices,
    ) -> Self {
        let orig = old_price.amount;
        let sale = final_price.amount;
        Prices {
            sale: if sale != orig { Some(sale) } else { None },
            original: orig,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TntPrice {
    amount: Decimal,
}
