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
let surfaceData = null;
let copiedTargets = null;
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
  el.classList.add("control"); el.dataset.role=role; el.dataset.id=control.id; el.setAttribute("tabindex","0");
  el.addEventListener("click",()=>select(role,control.id)); return el;
}

function select(role,id){ selected={role,id}; refresh(); document.querySelector(`.control[data-role="${role}"][data-id="${id}"]`)?.focus(); }
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
    message(`${result.bindings}개 버튼 검증 완료${apply?(invoke?" · 적용 및 저장됨":" · 브라우저에 저장됨"):""}`,true);
  }catch(error){message(String(error),false)}
}
function message(value,ok){const el=document.querySelector("#message");el.textContent=value;el.className=ok?"ok":"error"}

async function init(){
  document.querySelector("#mode").textContent=invoke?"TAURI 연결됨":"브라우저 데모";
  const target=document.querySelector("#target"); choices.forEach(value=>{const option=document.createElement("option");option.value=value.join(",");option.textContent=value.join(" + ");target.append(option)});
  const data=await surfaces(); surfaceData=data;
  const saved=invoke?await invoke("load_saved_layout"):JSON.parse(localStorage.getItem("piu-rise-layout")||"null");
  if(saved?.devices){for(const role of ["left","main"]) Object.assign(bindings[role],saved.devices[role]?.bindings||{})}
  for(const role of ["left","main"]){const svg=document.querySelector(`#${role}`);data[role].controls.forEach(control=>svg.append(elementFor(control,role)))}
  document.querySelector("#assign").onclick=()=>{if(!selected)return message("먼저 버튼을 선택하세요.",false);bindings[selected.role][selected.id]=target.value.split(",");refresh()};
  document.querySelector("#clear").onclick=()=>{if(selected){delete bindings[selected.role][selected.id];refresh()}};
  document.querySelector("#validate").onclick=()=>check(false);document.querySelector("#apply").onclick=()=>check(true);refresh();
  document.querySelector("#refresh-ports").onclick=loadPorts;
  document.querySelector("#start-controller").onclick=startController;
  document.querySelector("#stop-controller").onclick=stopController;
  for(const id of ["left-input","main-input"])document.querySelector(`#${id}`).onchange=event=>localStorage.setItem(`piu-rise-${id}`,event.target.value);
  document.addEventListener("keydown",handleKeyboard);
  await loadPorts(); await updateRuntimeStatus(); setInterval(updateRuntimeStatus,1000);
}
init().catch(error=>message(String(error),false));

function editableTarget(event){return ["INPUT","TEXTAREA","SELECT"].includes(event.target?.tagName)}
async function handleKeyboard(event){
  if(editableTarget(event)||!selected)return;
  if(event.key.startsWith("Arrow")){event.preventDefault();moveSelection(event.key);return}
  if(event.key==="Delete"||event.key==="Backspace"){event.preventDefault();delete bindings[selected.role][selected.id];refresh();return}
  if(event.ctrlKey&&event.key.toLowerCase()==="c"){
    event.preventDefault();copiedTargets=[...(bindings[selected.role][selected.id]||[])];
    if(navigator.clipboard&&copiedTargets.length)try{await navigator.clipboard.writeText(JSON.stringify(copiedTargets))}catch{}
    message(copiedTargets.length?"할당을 복사했습니다.":"복사할 할당이 없습니다.",copiedTargets.length>0);return;
  }
  if(event.ctrlKey&&event.key.toLowerCase()==="v"){
    event.preventDefault();let value=copiedTargets;
    if(navigator.clipboard){try{value=JSON.parse(await navigator.clipboard.readText())}catch{}}
    if(Array.isArray(value)&&value.length){bindings[selected.role][selected.id]=[...value];refresh();message("할당을 붙여넣었습니다.",true)}
    else message("붙여넣을 유효한 할당이 없습니다.",false);
  }
}

function center(geometry){
  if(geometry.type==="circle")return [geometry.cx,geometry.cy];
  if(geometry.type==="polygon"){const n=geometry.points.length;return geometry.points.reduce((v,p)=>[v[0]+p[0]/n,v[1]+p[1]/n],[0,0])}
  return [geometry.x+geometry.width/2,geometry.y+geometry.height/2];
}
function moveSelection(key){
  const controls=surfaceData[selected.role].controls,current=controls.find(c=>c.id===selected.id);if(!current)return;
  const [cx,cy]=center(current.geometry),horizontal=key==="ArrowLeft"||key==="ArrowRight",sign=(key==="ArrowLeft"||key==="ArrowUp")?-1:1;
  const candidates=controls.map(control=>{const [x,y]=center(control.geometry),primary=(horizontal?x-cx:y-cy)*sign,cross=Math.abs(horizontal?y-cy:x-cx);return {control,primary,cross}}).filter(v=>v.primary>.05).sort((a,b)=>(a.primary+a.cross*2)-(b.primary+b.cross*2));
  if(candidates[0]){select(selected.role,candidates[0].control.id);return}
  if(horizontal&&((selected.role==="left"&&sign>0)||(selected.role==="main"&&sign<0))){
    const role=selected.role==="left"?"main":"left",edge=surfaceData[role].controls.map(control=>{const [x,y]=center(control.geometry);return {control,x,y}}).sort((a,b)=>Math.abs(a.y-cy)-Math.abs(b.y-cy)||((sign>0?a.x:-a.x)-(sign>0?b.x:-b.x)))[0];
    if(edge)select(role,edge.control.id);
  }
}

async function loadPorts(){
  if(!invoke){document.querySelector("#runtime-status").textContent="브라우저 데모 · 실제 출력 없음";return}
  try{const ports=await invoke("list_midi_ports");for(const id of ["left-input","main-input"]){const el=document.querySelector(`#${id}`),old=el.value||localStorage.getItem(`piu-rise-${id}`);el.replaceChildren();ports.inputs.forEach(port=>{const option=document.createElement("option");option.value=port.index;option.textContent=`[${port.index}] ${port.name}`;el.append(option)});el.value=old}message(`${ports.inputs.length}개 MIDI 입력을 찾았습니다.`,true)}catch(error){message(String(error),false)}
}
async function startController(){
  if(!invoke)return message("브라우저 데모에서는 실제 컨트롤러를 시작하지 않습니다.",false);
  try{localValidate();const status=await invoke("start_controller",{leftInputIndex:Number(document.querySelector("#left-input").value),mainInputIndex:Number(document.querySelector("#main-input").value),layout:documentValue()});renderRuntimeStatus(status);message("컨트롤러가 실행 중입니다.",true)}catch(error){message(String(error),false)}
}
async function stopController(){if(!invoke)return;try{renderRuntimeStatus(await invoke("stop_controller"));message("모든 출력을 해제하고 중지했습니다.",true)}catch(error){message(String(error),false)}}
async function updateRuntimeStatus(){if(invoke)try{renderRuntimeStatus(await invoke("controller_status"))}catch(error){message(String(error),false)}}
function renderRuntimeStatus(status){document.querySelector("#runtime-status").textContent=status.running?"실행 중 · 일반 권한 출력":"중지됨";if(status.last_error)message(status.last_error,false)}
