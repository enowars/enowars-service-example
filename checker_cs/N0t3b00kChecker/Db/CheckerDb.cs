using EnoCore.Checker;
using Microsoft.Extensions.Logging;
using MongoDB.Driver;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace N0t3b00kChecker.Db
{
    public class CheckerDb
    {
        private readonly ILogger<CheckerDb> logger;
        private readonly IMongoCollection<NotebookUser> users;
        private readonly InsertOneOptions insertOneOptions = new() { BypassDocumentValidation = false };

        public CheckerDb(ILogger<CheckerDb> logger)
        {
            this.logger = logger;
            this.logger.LogDebug($"CheckerDb() via {MongoConnection})");
            var mongo = new MongoClient(MongoConnection);
            var db = mongo.GetDatabase("N0t3b00kCheckerDatabase");
            this.users = db.GetCollection<NotebookUser>("Users");
            this.users.Indexes.CreateOne(new CreateIndexModel<NotebookUser>(Builders<NotebookUser>.IndexKeys
                .Ascending(nu => nu.TaskChainId)));
        }

        public static string MongoHost => Environment.GetEnvironmentVariable("MONGO_HOST") ?? "localhost";

        public static string MongoPort => Environment.GetEnvironmentVariable("MONGO_PORT") ?? "27017";

        public static string MongoConnection => $"mongodb://{MongoHost}:{MongoPort}";

        public async Task InsertNotebookUser(NotebookUser user, CancellationToken token)
        {
            await this.users.InsertOneAsync(user, this.insertOneOptions, token);
        }

        public async Task<NotebookUser> FindNotebookUser(string taskChainId, CancellationToken token)
        {
            var cursor = await this.users.FindAsync(u => u.TaskChainId == taskChainId, cancellationToken: token);
            var user = await cursor.FirstOrDefaultAsync(token);
            if (user == null)
            {
                throw new MumbleException("Could not find old user");
            }

            return user;
        }
    }
}
