const fs=require('node:fs');
const path=require('node:path');
const sharp=require('sharp');
const {spawnSync}=require('node:child_process');
const root=path.resolve(__dirname,'../..');
const dir=path.join(root,'.cache/app-store/screenshots');
const slides=[
 {file:'translation-perfect',slug:'01-sentences',bg:'#F6F0E7',accent:'#D3EAD9',lines:['Put your words','to work.'],sub:['Practice real sentences.','Get feedback on your translations.']},
 {file:'home-cards-ready',slug:'02-daily-practice',bg:'#E6F0E6',accent:'#F9DCCA',lines:['A little practice.','Every day.'],sub:['Build your vocabulary','one review at a time.']},
 {file:'flashcard-written',slug:'03-vocabulary',bg:'#F8E6DD',accent:'#E2D6ED',lines:['Make new','words stick.'],sub:['Build vocabulary with flashcards','and spaced repetition.']},
 {file:'stats-overview',slug:'04-progress',bg:'#ECE5F1',accent:'#D5EBE0',lines:['See what','you’re learning.'],sub:['Follow your vocabulary,','reviews, and daily progress.']},
];
const esc=s=>s.replaceAll('&','&amp;').replaceAll('<','&lt;');
const args=process.argv.slice(2);
if(args.includes('--help')){
 console.log('Usage: pnpm generate [--no-build] [--simulator UUID] [--render-only]');
 process.exit(0);
}
const parityArgs=[];
let renderOnly=false;
for(let i=0;i<args.length;i++){
 if(args[i]==='--render-only') renderOnly=true;
 else if(args[i]==='--no-build') parityArgs.push(args[i]);
 else if(args[i]==='--simulator' && args[i+1]) parityArgs.push(args[i],args[++i]);
 else throw new Error('Unknown or incomplete argument: '+args[i]);
}
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
 const sw=864,sh=sw*source.height/source.width;
 if(sh+756>2778) throw new Error('Screenshot aspect ratio is too tall for the layout: '+s.file);
 const svg=`<svg xmlns="http://www.w3.org/2000/svg" width="1284" height="2778" viewBox="0 0 1284 2778">
 <defs><clipPath id="screen"><rect x="210" y="726" width="${sw}" height="${sh}" rx="99"/></clipPath><filter id="shadow" x="-50%" y="-15%" width="200%" height="140%"><feGaussianBlur stdDeviation="36"/></filter><linearGradient id="metal" x1="0" x2="1" y1="0" y2="0"><stop stop-color="#443744"/><stop offset=".35" stop-color="#C1B7BC"/><stop offset=".65" stop-color="#625661"/><stop offset="1" stop-color="#332936"/></linearGradient></defs>
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
 <image href="data:image/png;base64,${img}" x="210" y="726" width="${sw}" height="${sh}" clip-path="url(#screen)"/>
 </svg>`;
 fs.writeFileSync(path.join(dir,s.slug+'.svg'),svg);
 await sharp(Buffer.from(svg)).flatten({background:s.bg}).removeAlpha().png().toFile(path.join(dir,s.slug+'.png'));
 encoded.push(fs.readFileSync(path.join(dir,s.slug+'.png')).toString('base64'));
}
const w=1284/3,h=2778/3;
const board=`<svg xmlns="http://www.w3.org/2000/svg" width="${w*4}" height="${h}" viewBox="0 0 ${w*4} ${h}">${encoded.map((x,i)=>`<image href="data:image/png;base64,${x}" x="${i*w}" y="0" width="${w}" height="${h}"/>`).join('')}</svg>`;
await sharp(Buffer.from(board)).removeAlpha().png().toFile(path.join(dir,'preview.png'));
fs.writeFileSync(path.join(dir,'copy.json'),JSON.stringify(slides.map(({slug,lines,sub})=>({slug,headline:lines.join(' '),caption:sub.join(' ')})),null,2));
for (const s of slides){const m=await sharp(path.join(dir,s.slug+'.png')).metadata();console.log(s.slug,m.width,m.height,'alpha='+m.hasAlpha);}
})().catch(error=>{console.error(error);process.exitCode=1;});
