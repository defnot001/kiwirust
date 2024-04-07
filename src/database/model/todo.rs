use chrono::{DateTime, NaiveDateTime, Utc};
use serenity::all::UserId;
use sqlx::{prelude::FromRow, PgPool};

use crate::commands::todo::TodoChoice;

#[derive(Debug, FromRow)]
struct DbTodo {
    id: i32,
    title: String,
    #[sqlx(rename = "type")]
    todo_type: String,
    created_by: String,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Debug, FromRow)]
struct TodoTitle {
    title: String,
}

#[derive(Debug)]
pub struct Todo {
    pub id: i32,
    pub title: String,
    pub todo_type: TodoChoice,
    pub created_by: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct CreateTodo {
    title: String,
    todo_type: TodoChoice,
    created_by: UserId,
}

#[derive(Debug)]
pub struct TodoByType {
    pub survival: Vec<Todo>,
    pub creative: Vec<Todo>,
}

impl TryFrom<DbTodo> for Todo {
    type Error = anyhow::Error;

    fn try_from(db_todo: DbTodo) -> Result<Self, Self::Error> {
        if db_todo.id.is_negative() {
            anyhow::bail!("Todo ID cannot be negative")
        }

        if db_todo.title.is_empty() {
            anyhow::bail!("Todo title cannot be empty")
        }

        let todo_type = match db_todo.todo_type.as_str() {
            "survival" => TodoChoice::Survival,
            "creative" => TodoChoice::Creative,
            _ => anyhow::bail!(" \"{}\" is not a valid todo type", db_todo.todo_type),
        };

        let created_by = UserId::from(db_todo.created_by.parse::<u64>()?);

        Ok(Todo {
            id: db_todo.id,
            title: db_todo.title,
            created_at: db_todo.created_at.and_utc(),
            updated_at: db_todo.updated_at.and_utc(),
            todo_type,
            created_by,
        })
    }
}

impl CreateTodo {
    pub fn new(title: impl Into<String>, todo_type: TodoChoice, user_id: UserId) -> Self {
        Self {
            title: title.into(),
            created_by: user_id,
            todo_type,
        }
    }
}

pub struct TodoModelController;

impl TodoModelController {
    pub async fn create(db_pool: &PgPool, todo: CreateTodo) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO todos (title, type, created_by) VALUES ($1, $2, $3);")
            .bind(todo.title)
            .bind(todo.todo_type.to_string())
            .bind(todo.created_by.to_string())
            .execute(db_pool)
            .await?;

        Ok(())
    }

    pub async fn get_all(db_pool: &PgPool) -> anyhow::Result<TodoByType> {
        let db_todos = sqlx::query_as::<_, DbTodo>("SELECT * FROM todos;")
            .fetch_all(db_pool)
            .await?
            .into_iter()
            .map(Todo::try_from)
            .collect::<Vec<anyhow::Result<Todo>>>();

        let mut response: TodoByType = TodoByType {
            survival: Vec::new(),
            creative: Vec::new(),
        };

        for todo in db_todos {
            let todo = match todo {
                Ok(todo) => todo,
                Err(e) => anyhow::bail!("Error converting DbTodo to Todo: {e}"),
            };

            match todo.todo_type {
                TodoChoice::Survival => response.survival.push(todo),
                TodoChoice::Creative => response.creative.push(todo),
            }
        }

        Ok(response)
    }

    pub async fn all_titles(db_pool: &PgPool) -> Vec<String> {
        sqlx::query_as::<_, TodoTitle>("SELECT title FROM todos;")
            .fetch_all(db_pool)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(|db_title| db_title.title)
            .collect::<Vec<String>>()
    }

    pub async fn delete_by_title(db_pool: &PgPool, title: impl Into<String>) -> anyhow::Result<()> {
        if let Err(e) = sqlx::query("DELETE FROM todos WHERE title = $1;")
            .bind(title.into())
            .execute(db_pool)
            .await
        {
            Err(anyhow::anyhow!(e))
        } else {
            Ok(())
        }
    }

    pub async fn update_title(
        db_pool: &PgPool,
        old: impl Into<String>,
        new: impl Into<String>,
    ) -> anyhow::Result<()> {
        sqlx::query("UPDATE todos SET title = $1 WHERE title = $2;")
            .bind(new.into())
            .bind(old.into())
            .execute(db_pool)
            .await?;

        Ok(())
    }
}
