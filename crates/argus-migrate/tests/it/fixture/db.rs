//! The fixture database: the real schema, a row in every table for every board, every kv format.

use std::path::Path;

use rusqlite::{Connection, params};

use super::KEYS;

const KV_FORMATS: [&str; 16] = [
    "check-run:{}:r1",
    "check-seq:{}",
    "check-link:{}:r1",
    "check-flag:{}:r1",
    "launcher-played:{}:dev",
    "inbox-read:{}:coordinator",
    "run:{}",
    "stall-ack:{}:idle:I-1:99",
    "assigned-owner:{}:lead:5",
    "verify-owner:{}:lead:5",
    "wake-fault-owner:{}:lead:5",
    "changes:cleared:{}",
    "worktrees:made:{}",
    "worktrees:sent:{}",
    "worktrees:hygiene:{}",
    "worktrees:requests:{}",
];

fn scoped_rows(c: &Connection, k: &str) {
    c.execute("INSERT INTO docs(scope,coll,id,data,hash,seq,updated_at) VALUES (?1,'items','I-1','{}','h',1,'t')", [k]).unwrap();
    c.execute(
        "INSERT INTO changes(scope,coll,id,op,at) VALUES (?1,'items','I-1','set','t')",
        [k],
    )
    .unwrap();
    c.execute("INSERT INTO posts(id,scope,kind,source,body,status,received_at) VALUES (?1,?2,'note','cli','b','done','t')", params![format!("P-{k}"), k]).unwrap();
    c.execute(
        "INSERT INTO launch_defs(scope,name,def,hash,updated_at) VALUES (?1,'dev','{}','h','t')",
        [k],
    )
    .unwrap();
    c.execute("INSERT INTO wakes(id,n,scope,thread,reason,text,at,status) VALUES (?1,1,?2,'coordinator','r','t','t','queued')", params![format!("W-{k}"), k]).unwrap();
    c.execute(
        "INSERT INTO wake_faults(scope,thread,since,reason) VALUES (?1,'coordinator','t','r')",
        [k],
    )
    .unwrap();
}

fn run_rows(c: &Connection, k: &str) {
    let run = format!("run-{k}");
    c.execute("INSERT INTO runs(id,scope,rid,name,kind,status,cmd,cwd,origin,by,queued_at,instance) VALUES (?1,?2,'R-1','dev','run','exited','c','w','o','b','t','i')", params![run, k]).unwrap();
    c.execute(
        "INSERT INTO run_logs(run_id,seq,stream,at,text) VALUES (?1,1,'out','t','x')",
        [&run],
    )
    .unwrap();
    let path = format!("C:\\Users\\x\\.coord\\{k}\\proofs\\p1.png");
    c.execute("INSERT INTO attachments(id,scope,owner_kind,owner_id,n,path,mime,bytes,sha256,created_by,created_at) VALUES (?1,?2,'test','T-1',1,?3,'image/png',3,'s','x','t')", params![format!("A-{k}"), k, path]).unwrap();
}

fn project_rows(c: &Connection, k: &str) {
    c.execute(
        "INSERT INTO activity(at,kind,level,text,project) VALUES ('t','k','info','x',?1)",
        [k],
    )
    .unwrap();
    let url = format!("http://{k}.localhost/");
    c.execute("INSERT INTO notifications(at,level,title,text,project,by,toast,url) VALUES ('t','info','t','x',?1,'thread','shown',?2)", params![k, url]).unwrap();
    for format in KV_FORMATS {
        let key = format.replace("{}", k);
        c.execute("INSERT INTO kv(key,value) VALUES (?1,'1')", [key])
            .unwrap();
    }
}

/// Creates the database at `file` and fills it.
pub fn seed(file: &Path) {
    let conn = Connection::open(file).unwrap();
    for migration in argus_store::MIGRATIONS {
        conn.execute_batch(migration).unwrap();
    }
    for key in KEYS {
        scoped_rows(&conn, key);
        run_rows(&conn, key);
        project_rows(&conn, key);
    }
    for (key, value) in [("instance", "i-1"), ("changes:prunedTo", "5")] {
        conn.execute("INSERT INTO kv(key,value) VALUES (?1,?2)", [key, value])
            .unwrap();
    }
}
