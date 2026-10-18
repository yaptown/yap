const fs=require('node:fs');
const path=require('node:path');
const sharp=require('sharp');
const {spawnSync}=require('node:child_process');
const root=path.resolve(__dirname,'../..');
const slides=[
 {file:'translation-perfect',slug:'01-sentences',bg:'#F6F0E7',accent:'#D3EAD9',lines:['Put your words','to work.'],sub:['Practice real sentences.','Get feedback on your translations.']},
 {file:'home-cards-ready',slug:'02-daily-practice',bg:'#E6F0E6',accent:'#F9DCCA',lines:['A little practice.','Every day.'],sub:['Build your vocabulary','one review at a time.']},
 {file:'flashcard-written',slug:'03-vocabulary',bg:'#F8E6DD',accent:'#E2D6ED',lines:['Make new','words stick.'],sub:['Build vocabulary with flashcards','and spaced repetition.']},
 {file:'stats-overview',slug:'04-progress',bg:'#ECE5F1',accent:'#D5EBE0',lines:['See what','you’re learning.'],sub:['Follow your vocabulary,','reviews, and daily progress.']},
];
const esc=s=>s.replaceAll('&','&amp;').replaceAll('<','&lt;');
// Each device is one App Store screenshot size: its canvas, the simulator it
// captures on (none: the parity runner's default iPhone), and its framed layout.
const devices={
 iphone:{width:1284,height:2778,simulator:null,layout(s,img,source){
  const sw=864,sh=sw*source.height/source.width;
  if(sh+756>2778) throw new Error('Screenshot aspect ratio is too tall for the layout: '+s.file);
  return `<defs><clipPath id="screen"><rect x="210" y="726" width="${sw}" height="${sh}" rx="99"/></clipPath><filter id="shadow" x="-50%" y="-15%" width="200%" height="140%"><feGaussianBlur stdDeviation="36"/></filter><linearGradient id="metal" x1="0" x2="1" y1="0" y2="0"><stop stop-color="#443744"/><stop offset=".35" stop-color="#C1B7BC"/><stop offset=".65" stop-color="#625661"/><stop offset="1" stop-color="#332936"/></linearGradient></defs>
 <rect width="1284" height="2778" fill="${s.bg}"/>
 <circle cx="1220" cy="2010" r="710" fill="${s.accent}"/><circle cx="-70" cy="2720" r="510" fill="${s.accent}" opacity=".55"/>
 <g fill="#310634" font-family="Helvetica Neue,Helvetica,Arial,sans-serif">
 <text x="96" y="135" font-size="49" font-weight="700" letter-spacing="-1.5">Yap.Town<tspan fill="#F17D55">.</tspan></text>
 <text x="94" y="318" font-size="108" font-weight="700" letter-spacing="-4">${esc(s.lines[0])}</text><text x="94" y="432" font-size="108" font-weight="700" letter-spacing="-4">${esc(s.lines[1])}</text>
 <text x="98" y="536" font-size="40" fill="#66516A">${esc(s.sub[0])}</text><text x="98" y="590" font-size="40" fill="#66516A">${esc(s.sub[1])}</text>
 </g>
 <rect x="190" y="780" width="904" height="1900" rx="140" fill="#3F2243" opacity=".20" filter="url(#shadow)"/>
 <rect x="173" y="980" width="10" height="68" rx="5" fill="#60525D"/><rect x="173" y="1108" width="10" height="122" rx="5" fill="#60525D"/><rect x="173" y="1260" width="10" height="122" rx="5" fill="#60525D"/><rect x="1101" y="1160" width="10" height="174" rx="5" fill="#60525D"/>
 <rect x="180" y="696" width="924" height="${sh+60}" rx="129" fill="url(#metal)"/>
 <rect x="186" y="702" width="912" height="${sh+48}" rx="123" fill="#18141B" stroke="#746773" stroke-width="2"/>
 <image href="data:image/png;base64,${img}" x="210" y="726" width="${sw}" height="${sh}" clip-path="url(#screen)"/>`;
 }},
 // 13-inch iPad. The tablet is tall enough that it runs off the bottom edge,
 // which keeps the screen large; the app's content sits in its upper part.
 ipad:{width:2064,height:2752,simulator:'CD796BB4-5B2B-4E75-8F5A-36C10B1C04A2',layout(s,img,source){
  const sw=1760,sh=sw*source.height/source.width,x=(2064-sw)/2,y=860,b=44;
  return `<defs><clipPath id="screen"><rect x="${x}" y="${y}" width="${sw}" height="${sh}" rx="54"/></clipPath><filter id="shadow" x="-50%" y="-15%" width="200%" height="140%"><feGaussianBlur stdDeviation="54"/></filter><linearGradient id="metal" x1="0" x2="1" y1="0" y2="0"><stop stop-color="#443744"/><stop offset=".35" stop-color="#C1B7BC"/><stop offset=".65" stop-color="#625661"/><stop offset="1" stop-color="#332936"/></linearGradient></defs>
 <rect width="2064" height="2752" fill="${s.bg}"/>
 <circle cx="1960" cy="1900" r="1050" fill="${s.accent}"/><circle cx="-120" cy="2700" r="760" fill="${s.accent}" opacity=".55"/>
 <g fill="#310634" font-family="Helvetica Neue,Helvetica,Arial,sans-serif">
 <text x="${x}" y="190" font-size="64" font-weight="700" letter-spacing="-2">Yap.Town<tspan fill="#F17D55">.</tspan></text>
 <text x="${x-4}" y="420" font-size="150" font-weight="700" letter-spacing="-5.5">${esc(s.lines[0])}</text><text x="${x-4}" y="578" font-size="150" font-weight="700" letter-spacing="-5.5">${esc(s.lines[1])}</text>
 <text x="${x}" y="702" font-size="56" fill="#66516A">${esc(s.sub[0])} ${esc(s.sub[1])}</text>
 </g>
 <rect x="${x-b+20}" y="${y-b+70}" width="${sw+2*b-40}" height="${sh+2*b}" rx="110" fill="#3F2243" opacity=".20" filter="url(#shadow)"/>
 <rect x="${x+sw-260}" y="${y-b-9}" width="150" height="12" rx="6" fill="#60525D"/><rect x="${x+sw+b-3}" y="${y+150}" width="12" height="130" rx="6" fill="#60525D"/><rect x="${x+sw+b-3}" y="${y+310}" width="12" height="130" rx="6" fill="#60525D"/>
 <rect x="${x-b-6}" y="${y-b-6}" width="${sw+2*b+12}" height="${sh+2*b+12}" rx="${54+b+6}" fill="url(#metal)"/>
 <rect x="${x-b}" y="${y-b}" width="${sw+2*b}" height="${sh+2*b}" rx="${54+b}" fill="#18141B" stroke="#746773" stroke-width="2"/>
 <circle cx="1032" cy="${y-b/2}" r="7" fill="#2C2530"/>
 <image href="data:image/png;base64,${img}" x="${x}" y="${y}" width="${sw}" height="${sh}" clip-path="url(#screen)"/>`;
 }},
};
const args=process.argv.slice(2);
if(args.includes('--help')){
 console.log('Usage: pnpm generate [--device iphone|ipad] [--no-build] [--simulator UUID] [--render-only]');
 process.exit(0);
}
const parityArgs=[];
let renderOnly=false,deviceName='iphone',simulator=null;
for(let i=0;i<args.length;i++){
 if(args[i]==='--render-only') renderOnly=true;
 else if(args[i]==='--no-build') parityArgs.push(args[i]);
 else if(args[i]==='--simulator' && args[i+1]) simulator=args[++i];
 else if(args[i]==='--device' && devices[args[i+1]]) deviceName=args[++i];
 else throw new Error('Unknown or incomplete argument: '+args[i]);
}
const device=devices[deviceName];
simulator??=device.simulator;
if(simulator) parityArgs.push('--simulator',simulator);
const dir=path.join(root,'.cache/app-store/screenshots',deviceName);
fs.mkdirSync(path.join(dir,'sources'),{recursive:true});
if(!renderOnly){
 const capture=spawnSync('cargo',['xtask','parity','--ios-only','--only',slides.map(s=>s.file).join(','),'--out',path.join(dir,'sources'),...parityArgs],{cwd:root,stdio:'inherit'});
 if(capture.error) throw capture.error;
 if(capture.status!==0) process.exit(capture.status || 1);
}
const encoded=[];
(async()=>{
for(const s of slides){
 const img=fs.readFileSync(path.join(dir,'sources',s.file+'-ios.png')).toString('base64');
 const source=await sharp(Buffer.from(img,'base64')).metadata();
 const svg=`<svg xmlns="http://www.w3.org/2000/svg" width="${device.width}" height="${device.height}" viewBox="0 0 ${device.width} ${device.height}">
 ${device.layout(s,img,source)}
 </svg>`;
 fs.writeFileSync(path.join(dir,s.slug+'.svg'),svg);
 await sharp(Buffer.from(svg)).flatten({background:s.bg}).removeAlpha().png().toFile(path.join(dir,s.slug+'.png'));
 encoded.push(fs.readFileSync(path.join(dir,s.slug+'.png')).toString('base64'));
}
const w=device.width/3,h=device.height/3;
const board=`<svg xmlns="http://www.w3.org/2000/svg" width="${w*4}" height="${h}" viewBox="0 0 ${w*4} ${h}">${encoded.map((x,i)=>`<image href="data:image/png;base64,${x}" x="${i*w}" y="0" width="${w}" height="${h}"/>`).join('')}</svg>`;
await sharp(Buffer.from(board)).removeAlpha().png().toFile(path.join(dir,'preview.png'));
fs.writeFileSync(path.join(dir,'copy.json'),JSON.stringify(slides.map(({slug,lines,sub})=>({slug,headline:lines.join(' '),caption:sub.join(' ')})),null,2));
for (const s of slides){const m=await sharp(path.join(dir,s.slug+'.png')).metadata();console.log(s.slug,m.width,m.height,'alpha='+m.hasAlpha);}
})().catch(error=>{console.error(error);process.exitCode=1;});
