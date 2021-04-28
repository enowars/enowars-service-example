using MongoDB.Bson;
using MongoDB.Bson.Serialization.Attributes;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace N0t3b00kChecker.Db
{
    public class NotebookUser
    {
        [BsonId]
        [BsonRepresentation(BsonType.ObjectId)]
        public string? Id { get; set; }

        public string? Username { get; set; }

        public string? Password { get; set; }

        public string? NoteId { get; set; }

        public string? Note { get; set; }

        public string? TaskChainId { get; set; }
    }
}
