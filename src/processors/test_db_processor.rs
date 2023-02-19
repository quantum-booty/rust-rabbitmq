use anyhow::Result;
use sqlx::PgPool;
use tokio::time;
use tracing::info;

#[derive(Debug, sqlx::FromRow)]
#[allow(unused)]
struct Model {
    a: Option<i32>,
}

struct TestRepository {
    db: PgPool,
}

impl TestRepository {
    async fn get(&self) -> Result<Model> {
        let result = sqlx::query_as::<_, Model>("Select 1 a")
            .fetch_one(&self.db)
            .await?;
        Ok(result)
    }
}

pub async fn test_db_process(db: PgPool, wait_ms: u64) -> Result<()> {
    let test_repository = TestRepository { db };
    for _ in 0..500 {
        process(&test_repository).await?;
        time::sleep(time::Duration::from_millis(wait_ms)).await;
    }
    Ok(())
}

async fn process(test_repository: &TestRepository) -> Result<()> {
    let row = test_repository.get().await?;
    info!("{row:?}");
    Ok(())
}
