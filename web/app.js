const ACTIONS = [
  "P1_DOWN_LEFT", "P1_UP_LEFT", "P1_CENTER", "P1_UP_RIGHT", "P1_DOWN_RIGHT",
  "P2_DOWN_LEFT", "P2_UP_LEFT", "P2_CENTER", "P2_UP_RIGHT", "P2_DOWN_RIGHT"
];
const PAIRS = [
  ["P1_DOWN_LEFT","P1_CENTER"], ["P1_UP_LEFT","P1_CENTER"],
  ["P1_UP_RIGHT","P1_CENTER"], ["P1_DOWN_RIGHT","P1_CENTER"],
  ["P2_DOWN_LEFT","P2_CENTER"], ["P2_UP_LEFT","P2_CENTER"],
  ["P2_UP_RIGHT","P2_CENTER"], ["P2_DOWN_RIGHT","P2_CENTER"],
  ["P1_DOWN_RIGHT","P2_DOWN_LEFT"], ["P1_UP_RIGHT","P2_UP_LEFT"]
];
const choices = [...ACTIONS.map(a => [a]), ...PAIRS];
const bindings = { left: {}, main: {} };
let selected = null;
const invoke = window.__TAURI__?.core?.invoke;

function demoSurface() {
  const controls=[];
  for(let row=0;row<8;row++){
    for(let column=0;column<8;column++) controls.push({id:`grid.r${row}.c${column}`,geometry:{type:"rectangle",x:column,y:row,width:.86,height:.86},led:true});
    controls.push({id:`side.r${row}`,geometry:{type:"circle",cx:8.43,cy:row+.43,radius:.36},led:true});
  }
  return {model:"mk2",revision:1,width:8.9,height:8,controls};
}

async function surfaces(){
  if(invoke) return invoke("get_surface");
  return {left:demoSurface(),main:demoSurface()};
}

function elementFor(control, role){
  const ns="http://www.w3.org/2000/svg", g=control.geometry;
  const el=document.createElementNS(ns,g.type==="circle"?"circle":g.type==="polygon"?"polygon":"rect");
  if(g.type==="circle") ["cx","cy","r"].forEach(k=>el.setAttribute(k,k==="r"?g.radius:g[k]));
  else if(g.type==="polygon") el.setAttribute("points",g.points.map(p=>p.join(",")).join(" "));
  else ["x","y","width","height","rx"].forEach(k=>el.setAttribute(k,k==="rx" ? .12 : g[k]));
  el.classList.add("control"); el.dataset.role=role; el.dataset.id=control.id;
  el.addEventListener("click",()=>select(role,control.id)); return el;
}

function select(role,id){ selected={role,id}; refresh(); }
function refresh(){
  document.querySelectorAll(".control").forEach(el=>{
    const value=bindings[el.dataset.role][el.dataset.id]||[];
    el.classList.toggle("bound",value.length===1); el.classList.toggle("pair",value.length===2);
    el.classList.toggle("selected",selected?.role===el.dataset.role&&selected?.id===el.dataset.id);
  });
  document.querySelector("#selection").textContent=selected?`${selected.role} / ${selected.id}`:"없음";
  document.querySelector("#binding").textContent=selected?(bindings[selected.role][selected.id]||[]).join(" + ")||"미할당":"—";
}

function documentValue(){
  const devices={};
  for(const role of ["left","main"]) devices[role]={model:"mk2",surface_revision:1,bindings:bindings[role]};
  return {schema_version:1,id:"mk2-live-layout",name:"Mk2 Live Layout",devices};
}

function localValidate(){
  for(const role of ["left","main"]) for(const value of Object.values(bindings[role])){
    if(value.length<1||value.length>2) throw new Error("버튼에는 1개 또는 허용된 2개 액션만 할당할 수 있습니다.");
    if(value.length===2&&!PAIRS.some(pair=>pair.every(a=>value.includes(a)))) throw new Error(`허용되지 않은 조합: ${value.join(" + ")}`);
  }
  return {active_layout_id:"mk2-live-layout",bindings:Object.keys(bindings.left).length+Object.keys(bindings.main).length,persisted:false};
}

async function check(apply=false){
  try{
    const result=invoke?await invoke(apply?"apply_layout":"validate_layout",{layout:documentValue()}):localValidate();
    if(apply&&!invoke) localStorage.setItem("piu-rise-layout",JSON.stringify(documentValue()));
    message(`${result.bindings}개 버튼 검증 완료${apply?(invoke?" · 저장됨":" · 브라우저 데모에서는 저장되지 않음"):""}`,true);
  }catch(error){message(String(error),false)}
}
function message(value,ok){const el=document.querySelector("#message");el.textContent=value;el.className=ok?"ok":"error"}

async function init(){
  document.querySelector("#mode").textContent=invoke?"TAURI 연결됨":"브라우저 데모";
  const target=document.querySelector("#target"); choices.forEach(value=>{const option=document.createElement("option");option.value=value.join(",");option.textContent=value.join(" + ");target.append(option)});
  const data=await surfaces();
  const saved=invoke?await invoke("load_saved_layout"):JSON.parse(localStorage.getItem("piu-rise-layout")||"null");
  if(saved?.devices){for(const role of ["left","main"]) Object.assign(bindings[role],saved.devices[role]?.bindings||{})}
  for(const role of ["left","main"]){const svg=document.querySelector(`#${role}`);data[role].controls.forEach(control=>svg.append(elementFor(control,role)))}
  document.querySelector("#assign").onclick=()=>{if(!selected)return message("먼저 버튼을 선택하세요.",false);bindings[selected.role][selected.id]=target.value.split(",");refresh()};
  document.querySelector("#clear").onclick=()=>{if(selected){delete bindings[selected.role][selected.id];refresh()}};
  document.querySelector("#validate").onclick=()=>check(false);document.querySelector("#apply").onclick=()=>check(true);refresh();
}
init().catch(error=>message(String(error),false));
