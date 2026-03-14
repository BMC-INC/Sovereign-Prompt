import { Composition } from "remotion";
import { SovereignPromptVideo } from "./Video";
import { HowItWorksGraphic, OptimizationPipelineGraphic } from "./ReadmeGraphics";

export const RemotionRoot: React.FC = () => {
  return (
    <>
      <Composition
        id="SovereignPrompt"
        component={SovereignPromptVideo}
        durationInFrames={3130}
        fps={30}
        width={1920}
        height={1080}
      />
      <Composition
        id="HowItWorksGraphic"
        component={HowItWorksGraphic}
        durationInFrames={150}
        fps={24}
        width={1200}
        height={600}
      />
      <Composition
        id="OptimizationPipelineGraphic"
        component={OptimizationPipelineGraphic}
        durationInFrames={150}
        fps={24}
        width={1200}
        height={600}
      />
    </>
  );
};
