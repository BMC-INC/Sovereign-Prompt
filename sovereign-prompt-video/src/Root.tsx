import { Composition } from "remotion";
import { SovereignPromptVideo } from "./Video";

export const RemotionRoot: React.FC = () => {
  return (
    <Composition
      id="SovereignPrompt"
      component={SovereignPromptVideo}
      durationInFrames={570}
      fps={30}
      width={1920}
      height={1080}
    />
  );
};
