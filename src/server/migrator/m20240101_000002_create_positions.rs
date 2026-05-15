use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Positions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Positions::Id)
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Positions::UserId).text().not_null())
                    .col(ColumnDef::new(Positions::Latitude).double().not_null())
                    .col(ColumnDef::new(Positions::Longitude).double().not_null())
                    .col(ColumnDef::new(Positions::AltitudeM).integer().null())
                    .col(ColumnDef::new(Positions::SymbolTable).text().not_null())
                    .col(ColumnDef::new(Positions::SymbolCode).text().not_null())
                    .col(ColumnDef::new(Positions::Comment).text().null())
                    .col(
                        ColumnDef::new(Positions::DateCreated)
                            .timestamp()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Positions::Table, Positions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Positions::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Positions {
    Table,
    Id,
    UserId,
    Latitude,
    Longitude,
    AltitudeM,
    SymbolTable,
    SymbolCode,
    Comment,
    DateCreated,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}
