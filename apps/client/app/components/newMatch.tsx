import Radio from '../components/radio';
import { useState } from 'react';
import { Turtle, Clock, Zap } from 'lucide-react';
import { Button } from '~/components/ui/button';
import { Slider, SliderRange, SliderThumb, SliderTrack } from '@radix-ui/react-slider';


const NewMatch = () => {
  const [mode, setMode] = useState("r10");

  return (
    <div className="flex h-dvh max-w-screen-2xl items-center justify-center">
      <div className="">
        <div className="grid grid-cols-3 gap-4">
          <Radio fn={setMode} target="b1" value={mode}>
            <Clock size={20} className="mr-3" />
            1 min
          </Radio>
          <Radio fn={setMode} target="b11" value={mode}>
            <Clock size={20} className="mr-3" />
            1 | 1
          </Radio>
          <Radio fn={setMode} target="b21" value={mode}>
            <Clock size={20} className="mr-3" />
            2 | 1
          </Radio>
          <Radio fn={setMode} target="b3" value={mode}>
            <Zap size={20} className="mr-3" />
            3 min
          </Radio>
          <Radio fn={setMode} target="b32" value={mode}>
            <Zap size={20} className="mr-3" />
            3 | 2
          </Radio>
          <Radio fn={setMode} target="b5" value={mode}>
            <Zap size={20} className="mr-3" />
            5 min
          </Radio>
          <Radio fn={setMode} target="r10" value={mode}>
            <Turtle size={20} className="mr-3" />
            10 min
          </Radio>
          <Radio fn={setMode} target="r1510" value={mode}>
            <Turtle size={20} className="mr-3" />
            15 | 10
          </Radio>
          <Radio fn={setMode} target="r30" value={mode}>
            <Turtle size={20} className="mr-3" />
            30 min
          </Radio>
        </div>
        <div className="mt-4 flex justify-between items-center">
          <div className="h-2 w-52">
            <Slider className="flex items-center relative" name="value" defaultValue={[5]} max={500} step={5} orientation="horizontal">
              <SliderTrack className="relative h-2 flex-grow bg-black rounded-lg">
                <SliderRange className="absolute bg-black" />
              </SliderTrack>
              <SliderThumb className="block w-5 h-5 bg-white rounded-xl border-black border-2 outline-none" />
            </Slider>
          </div>
          <div>
            <Button className="w-64 bg-black text-white">Play</Button>
          </div>
        </div>
      </div>
    </div >
  );
};

export default NewMatch;
