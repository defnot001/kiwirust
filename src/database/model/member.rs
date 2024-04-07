use anyhow::Context;
use chrono::{DateTime, NaiveDateTime, Utc};
use poise::serenity_prelude as serenity;
use serenity::{Timestamp, UserId};
use sqlx::{prelude::FromRow, PgPool};
use uuid::Uuid;

pub struct MemberModelController;

#[derive(Debug, FromRow)]
struct DbMinecraftMember {
    discord_id: String,
    trial_member: bool,
    minecraft_uuids: Vec<Uuid>,
    member_since: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct MinecraftMember {
    pub discord_id: UserId,
    pub trial_member: bool,
    pub minecraft_uuids: Vec<Uuid>,
    pub member_since: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CreateMember {
    pub discord_id: UserId,
    pub trial_member: bool,
    pub minecraft_uuids: Vec<Uuid>,
    pub member_since: NaiveDateTime,
}

#[derive(Debug)]
pub struct UpdateMember {
    pub discord_id: UserId,
    pub trial_member: Option<bool>,
    pub minecraft_uuids: Option<Vec<Uuid>>,
    pub member_since: Option<NaiveDateTime>,
}

impl TryFrom<DbMinecraftMember> for MinecraftMember {
    type Error = anyhow::Error;

    fn try_from(db_member: DbMinecraftMember) -> Result<Self, Self::Error> {
        Ok(MinecraftMember {
            discord_id: UserId::from(db_member.discord_id.parse::<u64>()?),
            trial_member: db_member.trial_member,
            minecraft_uuids: db_member.minecraft_uuids,
            member_since: db_member.member_since.and_utc(),
            updated_at: db_member.updated_at.and_utc(),
        })
    }
}

impl MemberModelController {
    pub async fn get_by_id(
        db_pool: &PgPool,
        user_id: &UserId,
    ) -> anyhow::Result<Option<MinecraftMember>> {
        let maybe_db_member =
            sqlx::query_as::<_, DbMinecraftMember>("SELECT * FROM members WHERE discord_id = $1;")
                .bind(user_id.to_string())
                .fetch_optional(db_pool)
                .await?;

        if let Some(m) = maybe_db_member {
            Some(MinecraftMember::try_from(m)).transpose()
        } else {
            Ok(None)
        }
    }

    pub async fn get_all(db_pool: &PgPool) -> anyhow::Result<Vec<MinecraftMember>> {
        sqlx::query_as::<_, DbMinecraftMember>("SELECT * FROM members;")
            .fetch_all(db_pool)
            .await?
            .into_iter()
            .map(MinecraftMember::try_from)
            .collect()
    }

    pub async fn create(
        db_pool: &PgPool,
        create_member: CreateMember,
    ) -> anyhow::Result<MinecraftMember> {
        let query_result = sqlx::query_as::<_, DbMinecraftMember>(
            r#"
            INSERT INTO members
            (discord_id, trial_member, minecraft_uuids, member_since)
            VALUES ($1, $2, $3, $4)
            RETURNING *;
            "#,
        )
        .bind(create_member.discord_id.to_string())
        .bind(create_member.trial_member)
        .bind(create_member.minecraft_uuids)
        .bind(create_member.member_since)
        .fetch_one(db_pool)
        .await;

        match query_result {
            Ok(member) => member.try_into(),
            Err(e) => {
                let Some(db_error) = e.as_database_error() else {
                    return Err(anyhow::Error::from(e));
                };

                match db_error.kind() {
                    sqlx::error::ErrorKind::UniqueViolation => {
                        anyhow::bail!("Unique constraint violation")
                    }
                    _ => anyhow::bail!("Failed to insert"),
                }
            }
        }
    }

    pub async fn update(
        db_pool: &PgPool,
        update_member: UpdateMember,
    ) -> anyhow::Result<MinecraftMember> {
        let current_member = Self::get_by_id(db_pool, &update_member.discord_id.clone())
            .await
            .context("Failed to get member to update")?
            .context("Member cannot be updated because it doesn't exist")?;

        let updated_trial = update_member
            .trial_member
            .unwrap_or(current_member.trial_member);
        let updated_uuids = update_member
            .minecraft_uuids
            .unwrap_or(current_member.minecraft_uuids);
        let updated_member_since = update_member
            .member_since
            .unwrap_or(current_member.updated_at.naive_utc());
        let updated_at = Timestamp::now().naive_local();

        sqlx::query_as::<_, DbMinecraftMember>(
            r#"
            UPDATE members
            SET
                trial_member = $1,
                minecraft_uuids = $2,
                member_since = $3,
                update_at = $4,
            WHERE discord_id = $5
            RETURNING *;
            "#,
        )
        .bind(updated_trial)
        .bind(updated_uuids)
        .bind(updated_member_since)
        .bind(updated_at)
        .fetch_one(db_pool)
        .await?
        .try_into()
    }

    pub async fn delete(db_pool: &PgPool, user_id: &UserId) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM members WHERE discord_id = $1;")
            .bind(user_id.to_string())
            .execute(db_pool)
            .await
            .context(format!("Failed to delete member with id {}", user_id))?;

        Ok(())
    }
}
