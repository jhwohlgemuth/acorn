#!/usr/bin/env python3
"""Author a reference-based ORNL East Quad campus in approximate meters. MIT."""
import argparse
import importlib.util
import json
import math
from pathlib import Path

FINISHES = {
    "asphalt": ("ORNL campus asphalt", "#686A65", 0, 0.95),
    "bark": ("Oak bark", "#625447", 0, 0.94),
    "brick": ("ORNL campus brick", "#B77E68", 0, 0.9),
    "concrete": ("ORNL campus concrete", "#D2CDBE", 0, 0.88),
    "foliage-dark": ("Oak canopy shadow", "#344B27", 0, 0.95),
    "foliage-light": ("Oak canopy sun", "#76874B", 0, 0.9),
    "foliage-mid": ("Oak canopy green", "#526B38", 0, 0.92),
    "glass": ("ORNL campus glazing", "#658A87", 0.25, 0.17),
    "glass-dark": ("ORNL glazing shadow", "#344C50", 0.2, 0.2),
    "glass-light": ("ORNL glazing sky", "#A1B9B5", 0.2, 0.19),
    "lamp-white": ("Warm white lighting", "#EEE8D7", 0, 0.3),
    "lawn": ("ORNL campus lawn", "#829354", 0, 0.96),
    "metal": ("Campus graphite metal", "#4E5755", 0.7, 0.35),
    "metal-light": ("Brushed aluminum", "#BBC0BE", 0.65, 0.36),
    "mullion": ("Window aluminum", "#D8D9CF", 0.45, 0.36),
    "pond": ("ORNL pond water", "#537C72", 0.28, 0.14),
    "roof": ("Weathered roof membrane", "#C5C4B8", 0, 0.88),
    "roof-dark": ("Dark roof membrane", "#787A76", 0, 0.9),
    "stone": ("ORNL limestone trim", "#E1D8C6", 0, 0.79),
    "tire": ("Rubber tires", "#282C2B", 0, 0.94),
    "vehicle-blue": ("Campus vehicle blue", "#527185", 0.35, 0.28),
    "vehicle-silver": ("Campus vehicle silver", "#A3AAA8", 0.65, 0.3),
    "vehicle-white": ("Campus vehicle white", "#E9E9E0", 0.15, 0.27),
    "white-line": ("Weathered road white", "#D5D3BF", 0, 0.92),
}
SOURCES = [
    "https://www.ornl.gov/overview-ornl/",
    "https://www.ornl.gov/wp-content/uploads/2026/08/ORNL_Aerial_main-campus-scaled.jpg",
    "https://www.energy.gov/sites/default/files/2024-11/APPROVED%20SSAB%20Oak%20Ridge%20November%202024%20Meeting.pdf",
    "https://www.copearchitecture.com/project/ornl-visitors-conference-center/",
    "https://services2.arcgis.com/FiaPA4ga0iQKduv3/arcgis/rest/services/USA_Structures_View/FeatureServer/0",
]


def box(m, center, size, material, angle=0):
    w, h, d = (value / 2 for value in size)
    rings = [[(-w, y, -d), (w, y, -d), (w, y, d), (-w, y, d)] for y in [-h, h]]
    return m.part(m.G.ring_mesh(rings), material, center, rotation=(0, angle, 0))


def building(m, name, pixels, height, floors, roof="roof-dark", ribbon=False):
    points = [plan(*point) for point in pixels]
    parts = [slab(m, points, 0, height, "brick"), slab(m, points, height + 0.02, 0.16, roof)]
    area = sum(a[0] * b[1] - b[0] * a[1] for a, b in zip(points, points[1:] + points[:1]))
    for a, b in zip(points, points[1:] + points[:1]):
        dx, dz = b[0] - a[0], b[1] - a[1]
        length = math.hypot(dx, dz)
        angle = -math.atan2(dz, dx)
        normal = ((dz if area > 0 else -dz) / length, (-dx if area > 0 else dx) / length)
        x, z = ((a[i] + b[i]) / 2 for i in range(2))
        parts += [box(m, (x, height + 0.30, z), (length + 0.1, 0.6, 0.38), "stone", angle),
                  box(m, (x, 0.42, z), (length, 0.84, 0.25), "stone", angle)]
        columns = max(1, int((length - 2) / 3.5))
        for level in range(floors):
            y = 2.2 + level * ((height - 1.1) / floors)
            parts.append(box(m, (x, y - 1.48, z), (length, 0.16, 0.14), "stone", angle))
            for column in range(columns):
                distance = (column + 0.5) * length / columns
                px, pz = a[0] + dx * distance / length, a[1] + dz * distance / length
                width = min(2.25 if not ribbon else 3.15, length / columns - 0.5)
                glazing = ["glass", "glass-dark", "glass", "glass-light"][(column * 7 + level * 3) % 4]
                parts += [box(m, (px + normal[0] * 0.11, y, pz + normal[1] * 0.11), (width + 0.26, 2.65, 0.22), "stone", angle),
                          box(m, (px + normal[0] * 0.24, y, pz + normal[1] * 0.24), (width, 2.39, 0.06), glazing, angle),
                          box(m, (px + normal[0] * 0.28, y, pz + normal[1] * 0.28), (0.065, 2.4, 0.065), "mullion", angle),
                          box(m, (px + normal[0] * 0.28, y - 0.55, pz + normal[1] * 0.28), (width, 0.065, 0.065), "mullion", angle)]
    return named(parts, name)


def campus(m, landscape):
    # The public campus map defines plan relationships; heights and small details are estimated.
    b5100 = building(m, "5100-joint-institute", [(1775, 817), (1797, 817), (1797, 804), (1843, 804), (1843, 913), (1797, 913), (1797, 872), (1775, 872)], 9.5, 2, "roof")
    b5200 = building(m, "5200-research-support", [(1987, 793), (2010, 793), (2010, 804), (2034, 804), (2034, 834), (2047, 834), (2047, 863), (2034, 863), (2034, 911), (1994, 911), (1987, 902)], 10.2, 2)
    b5300 = building(m, "5300-science-technology", [(1677, 706), (1719, 706), (1719, 692), (1845, 692), (1845, 743), (1750, 743), (1750, 783), (1677, 783)], 16.2, 4)
    b5700 = building(m, "5700-research-office", [(1795, 943), (2026, 943), (2026, 987), (1932, 987), (1932, 997), (1865, 997), (1865, 987), (1795, 987)], 15.1, 3, "roof-dark", True)
    b5600 = building(m, "5600-computational-sciences", [(1768, 990), (1864, 990), (1864, 1038), (1932, 1038), (1932, 1054), (1907, 1054), (1907, 1104), (1800, 1104), (1800, 1077), (1768, 1077)], 13.4, 3, "roof")
    b5800 = building(m, "5800-engineering-technology", [(1932, 989), (2038, 989), (2038, 1099), (2025, 1099), (2025, 1085), (1993, 1085), (1993, 1104), (1907, 1104), (1907, 1054), (1932, 1054)], 13.4, 3, "roof")
    glass = curtain(m, (-56, 0, -11.25), 11, 14.7, 0) + curtain(m, (-15, 0, -11.25), 11, 14.7, 0)
    glass += curtain(m, (-1.8, 0, 29), 43, 8.8, -math.pi / 2)
    glass += curtain(m, (22.3, 0, 29),43,8.8,math.pi/2) + visitor_atrium(m)
    glass += curtain(m, (-73.2, 0, 29), 22, 8.8, math.pi / 2) + curtain(m, (-72.2, 0, 101), 23, 16.2, math.pi / 2)
    details = [part for x, z, h, width, depth in [(-80,-26,15.3,12,7),(-5,-26,15.3,12,7),(-77,-66,13.6,15,11),(-5,-70,13.6,12,10),(-86,32,9.7,17,6),(11,28,10.4,10,12),(-124,96,16.4,12,12)]
               for part in rooftop(m, x, z, h, width, depth)]
    details += [part for x,z,h in [(-105,-63,13.6),(-60,-63,13.6),(-20,-77,13.6),(-2,-55,13.6),(-135,90,16.4),(-107,100,16.4),(13,10,10.4)] for part in rooftop(m,x,z,h,5,4)]
    # Raised clerestories visible on the low Joint Institute roof.
    details += [box(m, (-89, 11.15, z), (18, 2.5, 3.8), "stone") for z in [16,40]]
    details += curtain(m, (-89,10,18),16,1.7,0) + curtain(m,(-89,10,42),16,1.7,0)
    details += [box(m, (-40, 4.8, -6.8), (13, 0.4, 6), "metal-light"),
                box(m, (-78, 4, 28), (5, 0.35, 12), "metal-light")]
    details += [box(m, (x,2.35,-5.3),(.18,4.7,.18),"metal") for x in [-46,-34]]
    details += [box(m,(x,4.4,z),(.6,8.8,.7),"brick") for x in [-2.1,22.6] for z in [9,16,23,30,37,44,50]]
    details += [box(m,(x,4.7,29),(.8,.3,43),"stone") for x in [-2.1,22.6]]
    details += [box(m,(-72.0,8.5,99),(1,17,3),"stone")]
    details += [box(m,(-95,y,88.3),(44,.55,.4),"stone") for y in [3.7,7.6,11.5]]
    details += [box(m,(x,6.7,88.3),(.5,13.5,.4),"stone") for x in [-115+i*4.5 for i in range(10)]]
    details += signage(m,-105,12.5,114.4,0,"ORNL",.8)
    details += [m.C((-42,16.8,-25),3.3,2.5,"stone",64),m.C((-42,18.1,-25),2.9,.08,"glass-dark",64)]
    return b5100 + b5200 + b5300 + b5700 + b5600 + b5800 + named(glass, "curtain-walls") + named(details, "architectural-details") + site(m, landscape)


def curtain(m, center, width, height, angle):
    x,y,z = center
    def point(a,b,c):
        return (x + a * math.cos(angle) + c * math.sin(angle), y + b, z - a * math.sin(angle) + c * math.cos(angle))
    parts = [box(m, point(0,height/2,0), (width,height,.4), "glass",angle)]
    parts += [box(m,point(-width/2+i*width/max(1,int(width/1.7)),height/2,.23),(.08,height+.15,.12),"mullion",angle) for i in range(int(width/1.7)+1)]
    parts += [box(m,point(0,i*height/max(1,int(height/1.5)),.24),(width,.09,.13),"mullion",angle) for i in range(int(height/1.5)+1)]
    parts += [box(m,point(a,height/2,.38),(.35,height+.6,.5),"stone",angle) for a in [-width/2,width/2]]
    return parts


def load(name):
    spec = importlib.util.spec_from_file_location(name.replace("-", "_"), Path(__file__).with_name(f"{name}.py"))
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1] / "assets/models")
    args = parser.parse_args()
    m, landscape = load("model-collection"), load("model-ornl-landscape")
    parts = campus(m, landscape)
    center = [(min(v[axis] for part in parts for v in part["vertices"])+max(v[axis] for part in parts for v in part["vertices"]))/2 for axis in [0,2]]
    # Move the two-meter soil cross section to ground zero without scaling the campus.
    parts = [dict(part, name=f"{part['group']}-{i:05}", vertices=[[round((x-center[0])*1.38,7),round(y+2,7),round((z-center[1])*1.38,7)] for x,y,z in part["vertices"]]) for i,part in enumerate(parts)]
    materials = {key: {"extras": {"brand":"natural finish","colorName":FINISHES[key][0],"srgbHex":FINISHES[key][1]},
                 "pbrMetallicRoughness":{"baseColorFactor":[*m.BRAND.linear(FINISHES[key][1]),1],"metallicFactor":FINISHES[key][2],"roughnessFactor":FINISHES[key][3]}}
                 for key in sorted({part["material"] for part in parts}) if key in FINISHES}
    materials |= {key:m.BRAND.material(key) for key in sorted({part["material"] for part in parts}) if key not in FINISHES}
    value = {"id":"research/ornl-campus","category":"research","label":"ORNL — East Quad Campus","revision":1,
             "materials":materials,"parts":parts,"references":SOURCES,
             "surfaces":{key:{"kind":kind,"uvScale":scale} for key,kind,scale in [("asphalt","asphalt",.25),("brick","brick",.25),("concrete","concrete",.25),("lawn","lawn",.06),("roof","roof",.25),("roof-dark","roof",.25)]},
             "provenance":"Original detailed reconstruction of ORNL's East Quad from public ORNL aerial photography and the 2024 DOE campus map. Six buildings (5100, 5200, 5300, 5600, 5700, 5800), pond and landscaped approaches. Approximate meter scale, estimated heights and facade/landscape detail; not photogrammetry, surveyed CAD, a georeferenced site model or the full reservation. Local campus-map orientation. See research/ornl-campus.md for sources and limitations."}
    target = args.root / "research/ornl-campus.model.json"
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(value,sort_keys=True,separators=(",",":"))+"\n")
    palette_path = args.root / "palette.json"
    palette = json.loads(palette_path.read_text())
    palette["naturalFinishes"] |= {value[0]:value[1] for value in FINISHES.values()}
    palette_path.write_text(json.dumps(palette,sort_keys=True,indent=2)+"\n")
    print(f"Authored {value['id']}: {len(parts)} parts, {sum(len(part['triangles']) for part in parts)} triangles")


def named(parts, group):
    return [dict(part,group=group) for part in parts]


def path(m, points, width, material="concrete", y=.08):
    closed = math.dist(points[0],points[-1]) < 1e-8
    points = points[:-1] if closed else points
    edges = list(zip(points,points[1:]+([points[0]] if closed else [])))
    normals = [(-(b[1]-a[1])/math.dist(a,b),(b[0]-a[0])/math.dist(a,b)) for a,b in edges]
    vertices = []
    for i,point in enumerate(points):
        before = normals[i-1] if i or closed else normals[0]
        after = normals[i%len(normals)] if closed or i<len(normals) else normals[-1]
        summed = (before[0]+after[0],before[1]+after[1])
        length = math.hypot(*summed)
        normal = (summed[0]/length,summed[1]/length)
        amount = width/2/max(.25,normal[0]*after[0]+normal[1]*after[1])
        vertices += [(point[0]+side*normal[0]*amount,y,point[1]+side*normal[1]*amount) for side in [-1,1]]
    faces = [face for i in range(len(edges)) for j in [(i+1)%len(points)] for face in [(2*i,2*i+1,2*j),(2*j,2*i+1,2*j+1)]]
    return [m.part((vertices,faces),material)]


def plan(x, y):
    return ((x-1990)*.5,(920-y)*.5)


def rooftop(m,x,z,y,width,depth):
    parts = [box(m,(x,y+.8,z),(width,1.6,depth),"metal-light"),box(m,(x,y+.08,z),(width+.6,.16,depth+.6),"metal")]
    for dx in [-width*.27,width*.27]:
        for dz in [-depth*.24,depth*.24]:
            r = min(width,depth)*.16
            parts += [m.C((x+dx,y+1.69,z+dz),r,.18,"metal",24),m.T((x+dx,y+1.79,z+dz),r,.045,"metal-light")]
            parts += [box(m,(x+dx,y+1.81,z+dz),(r*1.65,.04,.1),"metal-light",a*math.pi/3) for a in range(3)]
    parts += [box(m,(x,y+.35+i*.19,z+depth/2+.03),(width-.5,.055,.06),"metal") for i in range(6)]
    parts += [box(m,(x+width*.3,y+.4,z+depth*.9),(1,.65,depth),"metal-light")]
    return parts


def signage(m,x,y,z,angle,text,size):
    glyphs = {"0":[(0,0,0,1),(0,1,.6,1),(.6,1,.6,0),(.6,0,0,0)],
              "2":[(0,1,.6,1),(.6,1,.6,.5),(.6,.5,0,.5),(0,.5,0,0),(0,0,.6,0)],
              "5":[(.6,1,0,1),(0,1,0,.5),(0,.5,.6,.5),(.6,.5,.6,0),(.6,0,0,0)],
              "L":[(0,1,0,0),(0,0,.6,0)],"N":[(0,0,0,1),(0,1,.6,0),(.6,0,.6,1)],
              "O":[(0,0,0,1),(0,1,.6,1),(.6,1,.6,0),(.6,0,0,0)],
              "R":[(0,0,0,1),(0,1,.6,1),(.6,1,.6,.55),(.6,.55,0,.55),(.2,.55,.6,0)]}
    parts = [box(m,(x+(len(text)*.85-.25)*size/2,y+.5*size,z-.1),(len(text)*size*.85+.6,size*1.6,.25),"teal",angle)]
    for i,letter in enumerate(text):
        for ax,ay,bx,by in glyphs[letter]:
            parts.append(m.R((x+(i*.85+ax)*size,y+ay*size,z+.07),(x+(i*.85+bx)*size,y+by*size,z+.07),size*.045,"white"))
    return parts


def site(m,landscape):
    base = [box(m,(0,-1,12.5),(350,2,315),"bark"),box(m,(0,-.06,12.5),(350,.12,315),"lawn")]
    roads = [part for points,width in [([(-172,-6),(35,-6)],8),([(-122,-137),(-122,113),(-100,130)],7),([(-172,-100),(165,-100)],8),
             ([(3,170),(3,134)],9),([(20,114),(75,91),(135,79),(174,70)],9),([(2,95),(-10,75),(-10,-5)],7),
             ([(-13,121),(-38,130),(-75,147),(-145,160)],8),([(-121,70),(-75,70),(-75,106)],6),
             ([(-28,79),(-11,79)],6),([(-141,-42),(-122,-42)],5),
             ([(130,-38),(145,-38),(145,-100)],6)] for part in path(m,points,width,"asphalt",.07)]
    circle = [(3+18*math.cos(i*math.tau/96),114+18*math.sin(i*math.tau/96)) for i in range(97)]
    roads += path(m,circle,8,"asphalt",.07) + path(m,circle,0.22,"white-line",.2)
    base += [m.C((3,.1,114),13.6,.2,"brick",96),m.C((3,.21,114),11,.18,"lawn",96)]
    walks = path(m,[(-73,0),(-73,61),(-10,61)],3.2) + path(m,[(-73,0),(-10,0),(-10,61)],3.2)
    walks += path(m,[(-73,0),(-10,61)],1.8) + path(m,[(-73,61),(-10,0)],1.8) + path(m,[(-73,31),(-10,31)],2)
    walks += path(m,[(-72,15),(-9,15)],1.8)
    walks += path(m,[(-109,-10),(-109,-1),(-46,-1),(-46,-8)],3)
    pond = [plan(*p) for p in [(2058,862),(2091,854),(2119,852),(2141,838),(2151,808),(2169,798),(2210,800),(2272,815),(2300,832),(2308,861),(2305,918),(2294,927),(2060,927)]]
    base += [slab(m,pond,.025,.04,"pond")]
    walks += path(m,pond+[pond[0]],2.8,"stone",.18)
    # Narrow shoreline reeds and paving joints bring scale to the pond and quad.
    reeds = [m.R((x,.2,z),(x+.1,.65+(i%4)*.12,z+.15),.045,"foliage-mid") for i in range(75) for x,z in [(47+i*.82,58+4*math.sin(i*.13))] if 70<x<125]
    paving = [part for x in [-72+i*4 for i in range(16)] for part in path(m,[(x,-1.45),(x,1.45)],.035,"metal",.18)]
    parking = []
    for x,z,cols,rows in [(-45,79,12,2),(104,-38,18,2),(-149,-42,5,2)]:
        width,depth=cols*2.7+3,rows*11+4
        parking.append(box(m,(x,.05,z),(width,.1,depth),"asphalt"))
        for row in range(rows):
            for col in range(cols):
                px=x-width/2+2+col*2.7
                pz=z-depth/2+3+row*11
                parking += path(m,[(px-1.3,pz-2.5),(px-1.3,pz+2.5)],.1,"white-line",.17)
                if (col+row*3)%5 != 1:
                    car = landscape.car(m,px,pz,0 if row%2 else math.pi,["vehicle-silver","vehicle-white","vehicle-blue"][(col+row)%3])
                    parking += [dict(part,vertices=[(px+(a-px)/1.38,b,pz+(c-pz)/1.38) for a,b,c in part["vertices"]]) for part in car]
    trees = []
    placements = [(3,114,13),(-69,55,10),(-68,8,11),(-17,56,10),(-18,7,9),(-60,67,8),(-48,67,8)]
    placements += [(x,-109,10+(i%3)) for i,x in enumerate(range(-160,165,13))]
    placements += [(x,153+(i%2)*8,12+(i%4)) for i,x in enumerate(range(-165,173,11))]
    placements += [(158+(i%2)*8,z,12+(i%4)) for i,z in enumerate(range(-81,63,11))]
    placements += [(x,-126+(i%2)*7,11+(i%5)) for i,x in enumerate(range(-165,170,12))]
    placements += [(x,138,11+(i%3)) for i,x in enumerate(range(48,164,13))]
    placements += [(-160,z,11+(i%3)) for i,z in enumerate(range(11,135,14))]
    placements += [(35,73,12),(46,78,11),(145,65,13),(57,-12,10),(80,-15,10),(122,-14,11),(-115,4,10),(-115,33,11),(-115,62,10)]
    for i,(x,z,height) in enumerate(placements):
        trees += landscape.tree(m,x,z,height,i+30)
    furniture = [part for x,z in [(-68,3),(-68,60),(-14,3),(-14,60),(-41,3),(-41,60),(30,-10),(85,-12),(142,-12),(-117,-30),(-117,-70)] for part in landscape.lamp(m,x,z)]
    furniture += [part for x,z in [(-66,2),(-18,2),(-66,62),(-18,62)] for part in [box(m,(x,.55,z),(2.1,.12,.5),"bark"),box(m,(x,.9,z-.24),(2.1,.55,.1),"bark"),box(m,(x-.75,.25,z),(.12,.5,.45),"metal"),box(m,(x+.75,.25,z),(.12,.5,.45),"metal")]]
    return named(base,"site-ground-and-pond") + named(roads+paving,"roads-and-markings") + named(walks+reeds,"quad-and-shoreline") + named(parking,"parking-and-vehicles") + named(trees,"landscape-trees") + named(furniture,"site-furniture")


def slab(m, points, y, height, material):
    return m.E([(x,-z) for x,z in points],height,material,(0,y+height/2,0),(-math.pi/2,0,0))


def visitor_atrium(m):
    vertices = [(x,y,z) for x in [-2.5,11] for y,z in [(0,52),(0,69),(12.8,65),(12.8,52)]]
    faces = [(0,3,2),(0,2,1),(4,5,6),(4,6,7),(0,1,5),(0,5,4),(1,2,6),(1,6,5),(2,3,7),(2,7,6),(3,0,4),(3,4,7)]
    parts = [m.part((vertices,[(c,b,a) for a,b,c in faces]),"glass")]
    parts += [m.R((x,0,69.05),(x,12.9,65.02),.065,"mullion") for x in [-2.5+i*1.5 for i in range(10)]]
    parts += [m.R((-2.5,y,69.05-y*4/12.8),(11,y,69.05-y*4/12.8),.065,"mullion") for y in [i*1.6 for i in range(9)]]
    parts += [m.R((x,0,69),(x,12.9,65),.22,"stone") for x in [-2.5,11]]
    parts += [box(m,(4.25,12.9,58.5),(13.9,.25,13.5),"metal-light")]
    return parts


if __name__ == "__main__":
    main()
