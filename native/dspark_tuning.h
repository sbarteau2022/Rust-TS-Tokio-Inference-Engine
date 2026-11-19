#pragma once
#include <algorithm>
#include <cstddef>
#include <stdexcept>
#include <string>
#include <vector>
namespace native_dspark {
struct Tuning {
 size_t width=8;float confidence=-1.f;bool adaptive=false,confidence_explicit=false;int head=0;
 static Tuning parse(const std::vector<std::string>& args){
  Tuning t;bool seen_width=false,seen_conf=false,seen_adaptive=false,seen_head=false;
  for(const auto& arg:args){
   if(arg=="bf16"||arg=="checkpoint-head"||arg=="checkpoint-bf16"){
    if(seen_head)throw std::invalid_argument("duplicate head/precision");seen_head=true;t.head=arg=="bf16"?2:arg=="checkpoint-head"?1:3;
   }else if(arg.starts_with("--width=")){
    if(seen_width)throw std::invalid_argument("duplicate width");seen_width=true;auto value=arg.substr(8);
    if(value=="2")t.width=2;else if(value=="4")t.width=4;else if(value=="8")t.width=8;else throw std::invalid_argument("width must be2/4/8");
   }else if(arg.starts_with("--confidence=")){
    if(seen_conf)throw std::invalid_argument("duplicate confidence");seen_conf=true;t.confidence_explicit=true;auto value=arg.substr(13);
    if(value=="none")t.confidence=-1;else if(value=="0.8")t.confidence=.8f;else if(value=="0.9")t.confidence=.9f;else if(value=="0.95")t.confidence=.95f;else if(value=="0.98")t.confidence=.98f;else throw std::invalid_argument("unsupported confidence");
   }else if(arg=="--adaptive"){
    if(seen_adaptive)throw std::invalid_argument("duplicate adaptive");seen_adaptive=true;t.adaptive=true;
   }else throw std::invalid_argument("unknown DSpark tuning flag");
  }
  return t;
 }
 size_t initial()const{return adaptive?std::min<size_t>(4,width):width;}
 size_t next(size_t current,size_t admitted,size_t accepted)const{
  if((current!=2&&current!=4&&current!=8)||current>width||admitted>=current||accepted>admitted)throw std::invalid_argument("adaptive transition bounds");
  if(!adaptive)return width;
  if(!admitted||accepted<admitted)return std::max<size_t>(2,current/2);
  return std::min(width,current*2);
 }
};
}
