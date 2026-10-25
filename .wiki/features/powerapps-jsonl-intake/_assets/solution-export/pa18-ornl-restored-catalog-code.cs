using System;
using System.Collections.Generic;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Threading.Tasks;
using Newtonsoft.Json;
using Newtonsoft.Json.Linq;

public class Script : ScriptBase
{
    private const string Repository = "https://code.ornl.gov/api/v4/projects/22464/repository/";
    public override async Task<HttpResponseMessage> ExecuteAsync()
    {
        if (this.Context.OperationId != "GetProjectCatalog" && this.Context.OperationId != Convert.ToBase64String(Encoding.UTF8.GetBytes("GetProjectCatalog")))
            return await this.Context.SendAsync(this.Context.Request, this.CancellationToken).ConfigureAwait(false);
        try
        {
            var branch = await ReadAsync("branches/main");
            if (!branch.IsSuccessStatusCode) return Failure(branch.StatusCode, "Could not read the project repository.");
            var revision = (string)JObject.Parse(await branch.Content.ReadAsStringAsync())["commit"]?["id"];
            if (String.IsNullOrWhiteSpace(revision)) return Failure(HttpStatusCode.BadGateway, "Repository revision is missing.");
            var config = await ReadAsync("files/.acorn.jsonc?ref=" + Uri.EscapeDataString(revision));
            if (config.IsSuccessStatusCode)
            {
                var file = JObject.Parse(await config.Content.ReadAsStringAsync());
                if ((long?)file["size"] > 1048576) return Failure(HttpStatusCode.BadRequest, "Project configuration exceeds 1 MiB.");
                var text = new UTF8Encoding(false, true).GetString(Convert.FromBase64String((string)file["content"]));
                var document = JObject.Parse(text, new JsonLoadSettings { DuplicatePropertyNameHandling = DuplicatePropertyNameHandling.Error });
                if (document["schemaVersion"] != null && (string)document["schemaVersion"] != "2.0.0")
                    return Failure(HttpStatusCode.BadRequest, "Unsupported ACRE schemaVersion.");
                if (document.Property("folders") != null)
                    return Catalog(ConfigIds(document["folders"]), "config", revision);
            }
            else if (config.StatusCode != HttpStatusCode.NotFound)
                return Failure(config.StatusCode, "Could not read .acorn.jsonc. Discovery was not attempted.");
            var ids = new HashSet<string>(StringComparer.Ordinal);
            for (var page = 1; page <= 100; page++)
            {
                var response = await ReadAsync("tree?recursive=true&per_page=100&page=" + page + "&ref=" + Uri.EscapeDataString(revision));
                if (!response.IsSuccessStatusCode) return Failure(response.StatusCode, "Could not discover project folders.");
                var entries = JArray.Parse(await response.Content.ReadAsStringAsync());
                foreach (var entry in entries)
                {
                    var path = (string)entry["path"];
                    if ((string)entry["type"] == "blob" && (string)entry["mode"] != "120000" && path != null &&
                        ((string)entry["name"] == "index.json" || (string)entry["name"] == "index.yaml"))
                    {
                        var slash = path.LastIndexOf('/');
                        if (slash > 0) ids.Add(path.Substring(0, slash));
                    }
                }
                if (entries.Count < 100) return Catalog(ids, "discovery", revision);
            }
            return Failure(HttpStatusCode.BadRequest, "Repository discovery exceeded 10,000 entries; configure folders in .acorn.jsonc.");
        }
        catch (JsonException) { return Failure(HttpStatusCode.BadRequest, "Invalid project configuration or repository response."); }
        catch (FormatException) { return Failure(HttpStatusCode.BadRequest, "Invalid project configuration encoding."); }
        catch (ArgumentException) { return Failure(HttpStatusCode.BadRequest, "Invalid ACRE folder identifiers or aliases."); }
    }
    public static string[] ConfigIds(JToken folders)
    {
        if (folders == null || folders.Type == JTokenType.Null) throw new ArgumentException("folders is required when present");
        var entries = folders is JArray ? folders.Children().ToArray() : new[] { folders };
        var selectors = new HashSet<string>(StringComparer.Ordinal);
        var ids = new List<string>();
        foreach (var entry in entries)
        {
            if (entry.Type != JTokenType.String && entry.Type != JTokenType.Object) throw new ArgumentException("folder shape");
            var idToken = entry.Type == JTokenType.String ? entry : entry["id"];
            var id = Identifier(idToken);
            if (!selectors.Add(id)) throw new ArgumentException("duplicate selector");
            ids.Add(id);
            var aliases = entry.Type == JTokenType.Object ? entry["aliases"] : null;
            if (aliases != null && aliases.Type != JTokenType.Null)
            {
                foreach (var alias in aliases is JArray ? aliases.Children().ToArray() : new[] { aliases })
                    if (!selectors.Add(Identifier(alias))) throw new ArgumentException("duplicate alias");
            }
        }
        return ids.ToArray();
    }
    private static string Identifier(JToken token)
    {
        if (token == null || token.Type != JTokenType.String) throw new ArgumentException("identifier must be text");
        var id = (string)token;
        if (String.IsNullOrWhiteSpace(id) || id != id.Trim() || id.Any(Char.IsControl)) throw new ArgumentException("invalid identifier");
        return id;
    }
    private static HttpResponseMessage Catalog(IEnumerable<string> ids, string source, string revision)
    {
        var projects = new JArray(ids.OrderBy(id => id, StringComparer.OrdinalIgnoreCase).ThenBy(id => id, StringComparer.Ordinal)
            .Select(id => new JObject { ["id"] = id, ["label"] = id }));
        return new HttpResponseMessage(HttpStatusCode.OK) { Content = CreateJsonContent(new JObject
            { ["projects"] = projects, ["revision"] = revision, ["source"] = source }.ToString(Newtonsoft.Json.Formatting.None)) };
    }
    private static HttpResponseMessage Failure(HttpStatusCode status, string message)
    {
        return new HttpResponseMessage(status) { Content = CreateJsonContent(new JObject { ["message"] = message }.ToString(Newtonsoft.Json.Formatting.None)) };
    }
    private async Task<HttpResponseMessage> ReadAsync(string relative)
    {
        var request = new HttpRequestMessage(HttpMethod.Get, Repository + relative);
        foreach (var header in this.Context.Request.Headers)
            if (!String.Equals(header.Key, "Host", StringComparison.OrdinalIgnoreCase))
                request.Headers.TryAddWithoutValidation(header.Key, header.Value);
        return await this.Context.SendAsync(request, this.CancellationToken).ConfigureAwait(false);
    }
}
