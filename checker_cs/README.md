# n0t3b00k Checker (C#)

This is our example n0t3b00k checker implementation in C#.
A C# checker is a dotnet library, which is dynamically loaded and called when checker tasks arrive.

## Exported Interface
The library must export an classes that implement `ICheckerInitializer` and `IChecker`.
The implementation of `ICheckerInitializer` declares how many flag, noise and havoc variants the checker supports, specifies the service name which is used in log aggregations, and can add services to the the [service collection](https://docs.microsoft.com/en-ca/dotnet/core/extensions/dependency-injection).
This checker uses dependency injection to create database and notebook client objects, so it adds these types to the collection.
```c#
public int FlagVariants => 1;
public int NoiseVariants => 1;
public int HavocVariants => 3;
public string ServiceName => "n0t3b00k";
public void Initialize(IServiceCollection collection)
{
    collection.AddSingleton(typeof(CheckerDb));
    collection.AddTransient(typeof(NotebookClient));
}
```

The `IChecker` interface implementation handles the put and get methods.
The `CancellationToken` is canceled when the method takes too long, so we pass it to every asynchronous function call.
```c#
public async Task HandlePutFlag(CheckerTaskMessage task, CancellationToken token)
{
    switch (task.VariantId)
    {
        case 0:
            await DeployFlagToNote(task, token);
            break;
        default:
            throw new InvalidOperationException();
    }
}
```

## Persistent Storage
The `CheckerDb` class manages the MongoDb connection.
Since the checker creates many users during a CTF, the `CheckerDb` creates an index over the `TaskChainId`, which is used to find users during the getflag methods.
```c#
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
```
```c#
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
```

## EnoCore CheckerUtil
The `EnoCheckerTcpConnection` class is a wrapper around a TCP socket, which only throws Mumble- or OfflineExceptions.
Therefore, no try-catch blocks are required when calling methods of the class.
The receive functions are **not** thread-safe, do not call receive functions simultaneously on the same `EnoCheckerTcpConnection` object.
