using System;
using System.Collections.Generic;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Newtonsoft.Json.Linq;
public abstract class ScriptBase {
 public FakeContext Context {get;set;} = new FakeContext();
 public CancellationToken CancellationToken => CancellationToken.None;
 public static StringContent CreateJsonContent(string value) => new StringContent(value,Encoding.UTF8,"application/json");
 public abstract Task<HttpResponseMessage> ExecuteAsync();
}
public class FakeContext {
 public string OperationId {get;set;} = "GetProjectCatalog";
 public HttpRequestMessage Request {get;} = new HttpRequestMessage(HttpMethod.Get,"https://code.ornl.gov/api/v4/projects/22464/repository/tree");
 public Queue<HttpResponseMessage> Responses {get;} = new Queue<HttpResponseMessage>();
 public List<string> Paths {get;} = new List<string>();
 public Task<HttpResponseMessage> SendAsync(HttpRequestMessage request,CancellationToken ct) {Paths.Add(request.RequestUri.AbsoluteUri);return Task.FromResult(Responses.Dequeue());}
}
public class Program {
 static HttpResponseMessage Response(string s,HttpStatusCode code=HttpStatusCode.OK) => new HttpResponseMessage(code){Content=new StringContent(s)};
 static string File(string s) => new JObject{["size"]=Encoding.UTF8.GetByteCount(s),["content"]=Convert.ToBase64String(Encoding.UTF8.GetBytes(s))}.ToString();
 static void Check(bool value,string label){if(!value) throw new Exception(label); Console.WriteLine("PASS "+label);}
 static async Task<(HttpResponseMessage,FakeContext)> Run(string config,params HttpResponseMessage[] rest){var c=new FakeContext();c.Responses.Enqueue(Response("{\"commit\":{\"id\":\"abcdef\"}}"));c.Responses.Enqueue(config==null?Response("{}",HttpStatusCode.NotFound):Response(File(config)));foreach(var r in rest)c.Responses.Enqueue(r);return(await new Script{Context=c}.ExecuteAsync(),c);}
 public static async Task Main(){
 var a=await Run("{/* comment */\"schemaVersion\":\"2.0.0\",\"folders\":[\"zeta\",{\"id\":\"alpha\",\"aliases\":[\"a\"],\"path\":\"https://example.org/index.json\"},],}");
 var body=JObject.Parse(await a.Item1.Content.ReadAsStringAsync());Check(a.Item1.IsSuccessStatusCode && (string)body["projects"][0]["id"]=="alpha" && a.Item2.Paths.Count==2,"JSONC comments, trailing commas, mixed folders, canonical IDs and sorting");
 var b=await Run("{\"folders\":[]}");Check(JObject.Parse(await b.Item1.Content.ReadAsStringAsync())["projects"].Count()==0 && b.Item2.Paths.Count==2,"empty folders takes precedence");
 var c=await Run("{\"folders\":{\"id\":\"solo\"}}");Check(c.Item1.IsSuccessStatusCode,"singleton object");
 var d=await Run("{\"folders\":\"solo\"}");Check(d.Item1.IsSuccessStatusCode,"singleton string");
 foreach(var invalid in new[]{"{\"folders\":null}","{\"folders\":[\"a\",{\"id\":\"b\",\"aliases\":\"a\"}]}","{\"folders\":[{\"path\":\"a\"}]}","{broken}","{\"schemaVersion\":\"3.0.0\",\"folders\":[]}"}){var r=await Run(invalid);Check(!r.Item1.IsSuccessStatusCode&&r.Item2.Paths.Count==2,"invalid config fails without discovery");}
 var first=new JArray(Enumerable.Range(0,100).Select(i=>new JObject{["type"]="blob",["name"]="readme",["path"]="readme"+i}));
 var last="[{\"type\":\"blob\",\"name\":\"index.json\",\"path\":\"nested/a/index.json\"},{\"type\":\"blob\",\"name\":\"index.yaml\",\"path\":\"nested/a/index.yaml\"},{\"type\":\"blob\",\"name\":\"index.yaml\",\"path\":\"b/index.yaml\"}]";
 var e=await Run(null,Response(first.ToString()),Response(last));var eb=JObject.Parse(await e.Item1.Content.ReadAsStringAsync());Check(eb["projects"].Count()==2 && e.Item2.Paths.Last().Contains("page=2&ref=abcdef"),"pinned paginated discovery and duplicate-folder removal");
 var f=await Run("{}",Response("[]"));Check(f.Item1.IsSuccessStatusCode&&f.Item2.Paths.Count==3,"missing folders uses discovery");
 var ctx=new FakeContext();ctx.Responses.Enqueue(Response("{\"commit\":{\"id\":\"a\"}}"));ctx.Responses.Enqueue(Response("denied",HttpStatusCode.Forbidden));var g=await new Script{Context=ctx}.ExecuteAsync();Check(g.StatusCode==HttpStatusCode.Forbidden&&ctx.Paths.Count==2,"403 does not trigger discovery or retry");
 var pass=new FakeContext{OperationId="ReadLogbook"};pass.Responses.Enqueue(Response("original"));var h=await new Script{Context=pass}.ExecuteAsync();Check(await h.Content.ReadAsStringAsync()=="original"&&pass.Paths.Count==1,"existing operations pass through unchanged");
 }
}
