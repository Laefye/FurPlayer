import { useEffect, useRef } from 'react';
import { useEngine } from '../Engine';
import { Thumbnail } from './Thumbnail';

export function Player() {
    const { selectedAudio, state, engine, thumbnails } = useEngine();
    const audioRef = useRef<HTMLAudioElement>(null);

    useEffect(() => {
        if (selectedAudio && audioRef.current) {
            audioRef.current.src = selectedAudio[1];
            audioRef.current.play();
        }
    }, [selectedAudio]);

    return (
        <div className="bg-gray-800 p-4 rounded-xl flex items-center space-x-4">
            {selectedAudio ? (
                <>
                    {selectedAudio[0].id in thumbnails ? (
                        <Thumbnail className="w-16 h-16" src={thumbnails[selectedAudio[0].id]} />
                    ) : (
                        <div className="w-16 h-16 bg-gray-600 animate-pulse"></div>
                    )}
                    <div className="flex flex-col space-y-1">
                        <h2 className="text-lg font-bold">{selectedAudio[0].title}</h2>
                        <p className="text-gray-400">{selectedAudio[0].author}</p>
                    </div>
                    <audio ref={audioRef} controls className="ml-auto flex-grow">
                        Your browser does not support the audio element.
                    </audio>
                </>
            ) : (
                <p className="text-gray-400">Select an audio to play</p>
            )}
            {state === 'loading_audio' && <p className="text-gray-400">Loading...</p>}
        </div>
    );
}