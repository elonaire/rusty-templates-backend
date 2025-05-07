use crate::utils;

// should match the package name in the .proto file
pub mod acl_service {
    include!("out/acl.rs");
}

// should match the package name in the .proto file
pub mod email_service {
    include!("out/email.rs");
}

// should match the package name in the .proto file
pub mod files_service {
    include!("out/files.rs");
}

pub mod products_service {
    include!("out/products.rs");
}

pub mod orders_service {
    include!("out/orders.rs");
}

pub mod payments_service {
    include!("out/payments.rs");
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
