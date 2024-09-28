import { AnimatePresence, motion } from "framer-motion";
import { useState, useEffect } from "react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

import { invoke } from "@tauri-apps/api/tauri";

export const SetupPage = ({ startup, loadRecent, setupCheck }) => {
  const [showWelcome, setShowWelcome] = useState(true);
  const [showCard, setShowCard] = useState(false);
  const [rootFolder, setRootFolder] = useState(""); // State to hold the root folder input value
  const [extensions, setExtensions] = useState();

  const commonExt = [
    "doc",
    "docx",
    "pages",
    "rtf",
    "txt",
    "csv",
    "numbers",
    "xls",
    "xlsx",
    "html",
    "mp3",
    "mp4",
    "pptx",
  ];

  useEffect(() => {
    // Hide the "Welcome" message after 1.35 seconds
    const welcomeTimer = setTimeout(() => {
      setShowWelcome(false);
    }, 1350);

    // Show the card after the "Welcome" message fades out
    const cardTimer = setTimeout(() => {
      setShowCard(true);
    }, 1350 + 500); // Wait for welcome animation + fade-out duration

    // Cleanup timers on unmount
    return () => {
      clearTimeout(welcomeTimer);
      clearTimeout(cardTimer);
    };
  }, []);

  // Function to handle the form submission
  const setupData = async (e) => {
    try {
      await invoke("save_setup_file", {
        rootFolder,
        extensions,
      })
        .then(startup)
        .then(loadRecent)
        .then(setupCheck);
    } catch (error) {
      console.error("Error indexing:", error);
    }
  };

  return (
    <main className="h-dvh flex flex-col justify-center items-center">
      {/* Welcome Message */}
      <AnimatePresence>
        {showWelcome && (
          <motion.span
            className="text-6xl"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.5 }} // Controls fade-in and fade-out duration
          >
            Welcome
          </motion.span>
        )}
      </AnimatePresence>

      {/* Card fades in after the Welcome message fades out */}
      <AnimatePresence>
        {showCard && (
          <motion.div
            className="w-[65%] max-w-[500px]"
            initial={{ opacity: 0, y: 20 }} // Start off-screen slightly
            animate={{ opacity: 1, y: 0 }} // Fade in and move to position
            transition={{ duration: 0.5 }} // Controls the fade-in duration
          >
            <Card>
              <CardHeader>
                <CardTitle className="text-2xl">Setup your Finder</CardTitle>
                <CardDescription>
                  Set the root folder and preferred extensions
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="grid w-full items-center gap-4">
                  <div className="flex flex-col spacey-y-1.5">
                    <form>
                      <Label htmlFor="root-folder">Root Folder:</Label>
                      {/* Input field controlled by rootFolder state */}
                      <Input
                        className="px-2"
                        id="root-folder"
                        placeholder="Enter the root folder here"
                        value={rootFolder}
                        onChange={(e) => setRootFolder(e.target.value)} // Update the state with input value
                      />
                      <Label htmlFor="root-folder">Choose extensions</Label>
                      <ToggleGroup
                        type="multiple"
                        size={"lg"}
                        className="grid grid-cols-5"
                        value={extensions}
                        onValueChange={(extensions) => {
                          if (extensions) setExtensions(extensions);
                          console.log(extensions);
                        }}
                      >
                        {commonExt.map((ext) => (
                          <ToggleGroupItem key={ext} value={ext}>
                            {ext}
                          </ToggleGroupItem>
                        ))}
                      </ToggleGroup>
                      <Input
                        className="px-2"
                        id="extensions"
                        placeholder="Enter additional extensions"
                        // value={}
                        // onChange={(e) => setRootFolder(e.target.value)} // Update the state with input value
                      />
                    </form>
                  </div>
                </div>
              </CardContent>
              <CardFooter>
                {/* Button triggers form submission */}
                <Button className="w-full" onClick={setupData}>
                  Submit
                </Button>
              </CardFooter>
            </Card>
          </motion.div>
        )}
      </AnimatePresence>
    </main>
  );
};
