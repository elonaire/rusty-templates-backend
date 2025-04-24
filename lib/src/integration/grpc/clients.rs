use crate::utils;

// should match the package name in the .proto file
pub mod acl_service {
    tonic::include_proto!("acl");
}

// should match the package name in the .proto file
pub mod email_service {
    tonic::include_proto!("email");
}

// should match the package name in the .proto file
pub mod files_service {
    tonic::include_proto!("files");
}

pub mod products_service {
    tonic::include_proto!("products");
}

pub mod orders_service {
    tonic::include_proto!("orders");
}

pub mod payments_service {
    tonic::include_proto!("payments");
}

impl From<payments_service::UserPaymentDetails> for utils::models::UserPaymentDetails {
    fn from(user: payments_service::UserPaymentDetails) -> Self {
        Self {
            email: user.email,
            amount: user.amount,
            reference: user.reference,
        }
    }
}

impl From<utils::models::ProductSkuPrice> for products_service::ProductSkuPrice {
    fn from(product_sku_price: utils::models::ProductSkuPrice) -> Self {
        Self {
            product_sku: product_sku_price.product_sku,
            unit_price: product_sku_price.unit_price,
        }
    }
}
