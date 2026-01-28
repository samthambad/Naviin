use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(AppState::Table)
                    .if_not_exists()
                    .col(pk_auto(AppState::Id))
                    .col(decimal(AppState::CashBalance))
                    .col(big_integer(AppState::UpdatedAt))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Holding::Table)
                    .if_not_exists()
                    .col(pk_auto(Holding::Id))
                    .col(string(Holding::Symbol))
                    .col(decimal(Holding::Quantity))
                    .col(decimal(Holding::AvgCost))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Trade::Table)
                    .if_not_exists()
                    .col(pk_auto(Trade::Id))
                    .col(string(Trade::Symbol))
                    .col(decimal(Trade::Quantity))
                    .col(decimal(Trade::PricePer))
                    .col(string(Trade::Side))
                    .col(string(Trade::OrderType))
                    .col(big_integer(Trade::Timestamp))
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(OpenOrder::Table)
                    .if_not_exists()
                    .col(pk_auto(OpenOrder::Id))
                    .col(string(OpenOrder::OrderType))
                    .col(string(OpenOrder::Symbol))
                    .col(decimal(OpenOrder::Quantity))
                    .col(decimal(OpenOrder::Price))
                    .col(big_integer(OpenOrder::Timestamp))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Watchlist::Table)
                    .if_not_exists()
                    .col(pk_auto(Watchlist::Id))
                    .col(string(Watchlist::Symbol))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

}

#[derive(DeriveIden)]
enum AppState {
    Table,
    Id,
    CashBalance,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Holding {
    Table,
    Id,
    Symbol,
    Quantity,
    AvgCost,
}

#[derive(DeriveIden)]
enum Trade {
    Table,
    Id,
    Symbol,
    Quantity,
    PricePer,
    Side,
    OrderType,
    Timestamp,
}
#[derive(DeriveIden)]
enum OpenOrder {
    Table,
    Id,
    OrderType,
    Symbol,
    Quantity,
    Price,
    Timestamp,
}

#[derive(DeriveIden)]
enum Watchlist {
    Table,
    Id,
    Symbol,
}