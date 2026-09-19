#!/usr/bin/env python3
"""Restricted local operator tool; never expose via HTTP. Python standard library only."""
import argparse, json, os, sqlite3, pathlib, sys, time
os.umask(0o077)
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--db',default=os.environ.get('INTAKE_DB','private/intake.sqlite3'))
s=p.add_subparsers(dest='command',required=True)
s.add_parser('status')
s.add_parser('list')
a=s.add_parser('show');a.add_argument('id')
a=s.add_parser('delete');a.add_argument('id')
a=s.add_parser('retry');a.add_argument('id')
a=s.add_parser('stage');a.add_argument('id');a.add_argument('stage',choices=['received','conversation','matched','agreed','accepted','closed'])
a=s.add_parser('backup');a.add_argument('destination')
a=s.add_parser('check-backup');a.add_argument('source')
args=p.parse_args()
source=pathlib.Path(args.source if args.command=='check-backup' else args.db).resolve()
if not source.is_file(): sys.exit('Database does not exist; refusing to create an empty one.')
db=sqlite3.connect(source.as_uri()+'?mode='+('ro' if args.command in ['status','list','show','backup','check-backup'] else 'rw'),uri=True)
db.execute('PRAGMA foreign_keys=ON');db.row_factory=sqlite3.Row
if args.command=='status':
    for table,column in [('inquiries','stage'),('outbox','state')]:
        print(table, json.dumps([dict(r) for r in db.execute(f'SELECT {column},count(*) AS count FROM {table} GROUP BY {column}')]))
elif args.command=='list':
    for row in db.execute('SELECT i.id,i.kind,i.created,i.stage,o.state AS notification FROM inquiries i JOIN outbox o ON i.id=o.inquiry_id ORDER BY created DESC LIMIT 100'): print(json.dumps(dict(row)))
elif args.command=='show':
    row=db.execute('SELECT * FROM inquiries WHERE id=?',[args.id]).fetchone()
    if row is None:sys.exit('Inquiry not found')
    record=dict(row);record['payload']=json.loads(record['payload']);print(json.dumps(record,ensure_ascii=False,indent=2))
elif args.command=='delete':
    with db: count=db.execute('DELETE FROM inquiries WHERE id=?',[args.id]).rowcount
    print(f'Deleted {count} inquiry. Follow the backup deletion/restore procedure for retained copies.')
elif args.command=='retry':
    with db:count=db.execute("UPDATE outbox SET state='pending',attempts=0,next_attempt=0,last_error=NULL WHERE inquiry_id=? AND state='failed'",[args.id]).rowcount
    print(f'Requeued {count} failed notification. Captured and sent notifications are not resent.')
elif args.command=='stage':
    with db:count=db.execute('UPDATE inquiries SET stage=? WHERE id=?',[args.stage,args.id]).rowcount
    print(f'Updated {count} inquiry.')
elif args.command=='backup':
    dest=pathlib.Path(args.destination).resolve()
    if dest==source or dest.exists():sys.exit('Choose a new backup path; refusing to overwrite existing data.')
    dest.parent.mkdir(parents=True,exist_ok=True)
    with sqlite3.connect(dest) as target:db.backup(target)
    dest.chmod(0o600);print('Consistent SQLite backup created. Transfer it to the approved independent backup store.')
elif args.command=='check-backup':
    result=db.execute('PRAGMA integrity_check').fetchone()[0]
    print('Integrity:',result)
    print('Inquiries:',db.execute('SELECT count(*) FROM inquiries').fetchone()[0])
    print('Jobs:',db.execute('SELECT count(*) FROM outbox').fetchone()[0])
    if result!='ok':sys.exit(1)
